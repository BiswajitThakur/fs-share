//! # Sender Runtime
//!
//! This module implements the sender-side runtime logic for `fs-share`.
//!
//! ## Flow
//!
//! 1. Resolve receiver address:
//!    - Use CLI-provided address OR
//!    - Discover via UDP broadcast
//! 2. Establish TCP connection
//! 3. Upgrade stream (e.g., encryption/handshake)
//! 4. Send files to peer
//! 5. Receive files from peer
//!
use std::{
    borrow::Cow,
    fmt::Display,
    io::{self, Read, Write},
    net::SocketAddr,
    path::Path,
    sync::mpsc::Receiver,
    thread::JoinHandle,
};

use anyhow::Context;

use crate::{
    broadcast::receiver::{BroadcastReceiver, PayloadReader},
    pb::ProgressBar,
    tf::{sender_receive_file, sender_send_file},
};

/// Trait for data received from broadcast discovery.
///
/// This allows extracting the receiver's address from parsed payload.
pub trait ReceiverData {
    /// Returns the receiver's socket address
    fn addr(&self) -> SocketAddr;
}

/// Application abstraction for sender runtime.
///
/// Allows customization of:
/// - authentication
/// - stream upgrade (encryption/handshake)
/// - progress bar
/// - receiver selection
pub trait App {
    /// Raw stream type (e.g., TcpStream)
    type Stream: Read + Write;

    /// Upgraded stream (e.g., encrypted stream)
    type UpgradeStream: Read + Write;

    /// Broadcast prefix used for discovery filtering
    fn prefix(&self) -> &str;

    /// Broadcast address (UDP)
    fn broadcast_addr(&self) -> SocketAddr;

    /// Optional receiver address
    fn receiver_addr(&self) -> Option<SocketAddr>;

    /// Directory for saving received files
    fn download_dir<'a>(&'a self) -> Cow<'a, Path>;

    /// Authenticate connection (default: accept all)
    fn auth(&self, stream: &mut Self::Stream) -> io::Result<bool> {
        let _ = stream;
        Ok(true)
    }

    /// Upgrade stream (e.g., encryption/handshake)
    fn upgrade_stream(&self) -> impl Fn(Self::Stream) -> anyhow::Result<Self::UpgradeStream>;

    /// Create progress bar
    fn create_progress_bar(&self, total: u64) -> Box<dyn ProgressBar>;

    /// Select receiver address from discovered broadcast data
    fn select_receiver_addr<U>(
        &self,
        discovery: (
            Box<dyn FnOnce() + Send>,
            Receiver<(SocketAddr, U)>,
            JoinHandle<()>,
        ),
    ) -> Option<SocketAddr>
    where
        U: Clone + Display + PartialEq + ReceiverData + Send + 'static;
}

/// Run sender runtime.
///
/// Handles:
/// - receiver discovery
/// - connection
/// - file transfer (send + receive)
pub fn run_v1<A, P, ConnectFn, R>(
    app: A,
    files_to_send: impl Iterator<Item = P>,
    connect: ConnectFn,
) -> anyhow::Result<()>
where
    A: App,
    P: AsRef<Path>,
    ConnectFn: Fn(&A, SocketAddr) -> anyhow::Result<A::Stream>,
    R: for<'a> TryFrom<(SocketAddr, PayloadReader<'a>)>
        + ReceiverData
        + Clone
        + Display
        + PartialEq
        + Send
        + 'static,
{
    // Resolve receiver address
    let receiver_addr = match app.receiver_addr() {
        Some(addr) => Some(addr),
        None => {
            let receiver = BroadcastReceiver::builder()
                .prefix(app.prefix())
                .bind_addr(app.broadcast_addr())
                .buffer_size(4 * 1024)
                .build()
                .context("Failed to build BroadcastReceiver")?;

            let discovery = receiver.start::<R>();
            app.select_receiver_addr(discovery)
        }
    };

    let receiver_addr = receiver_addr.context("No valid receiver address found via broadcast")?;

    // Establish connection
    let stream = connect(&app, receiver_addr)
        .with_context(|| format!("Failed to connect to {}", receiver_addr))?;

    // Upgrade stream
    let mut stream = app.upgrade_stream()(stream)?;

    // Send files
    for path in files_to_send {
        sender_send_file(&app, path, &mut stream)?;
    }

    // Signal end of sending
    stream.write_all(b":eof:")?;
    stream.flush()?;

    // Receive files
    loop {
        let mut marker = [0u8; 5];
        stream.read_exact(&mut marker)?;

        match &marker {
            b":fff:" => {
                sender_receive_file(&app, &mut stream)?;
            }
            b":eof:" => break,
            _ => unreachable!("Invalid protocol marker"),
        }
    }

    Ok(())
}
