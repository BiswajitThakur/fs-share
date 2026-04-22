//! # Receiver Runtime Core
//!
//! This module provides the core runtime logic for handling incoming
//! connections and performing bidirectional file transfer.
//!
//! ## Flow
//!
//! 1. Create listener (TCP)
//! 2. Optionally start UDP broadcaster (for discovery)
//! 3. Accept and authenticate connection
//! 4. Upgrade stream (e.g., encryption/handshake)
//! 5. Receive files from peer
//! 6. Send files to peer
//!
//!
//! ## Design
//!
//! The [`App`] trait allows customizing:
//! - Authentication
//! - Broadcast discovery
//! - Progress bar
//! - Stream upgrade (e.g., encryption)

use std::{
    borrow::Cow,
    io::{self, Read, Write},
    net::SocketAddr,
    path::Path,
};

use anyhow::Context;

use crate::{
    pb::ProgressBar,
    tf::{receiver_receive_file, receiver_send_file},
};

/// Application abstraction for receiver runtime.
///
/// Implement this trait to customize networking behavior.
pub trait App {
    /// Raw stream type (e.g., TcpStream)
    type Stream: Read + Write;

    /// Upgraded stream (e.g., encrypted stream)
    type UpgradeStream: Read + Write;

    /// Prefix used for broadcast discovery
    fn prefix(&self) -> &str;

    /// UDP broadcast address
    fn broadcast_addr(&self) -> SocketAddr;

    /// Directory where received files will be stored
    fn download_dir<'a>(&'a self) -> Cow<'a, Path>;

    /// Disable broadcaster
    fn disable_broadcaster(&self) -> bool {
        false
    }
    /// Pre-process an incoming connection.
    ///
    /// This method is called immediately after a connection is accepted,
    /// and before authentication or stream upgrade.
    ///
    /// Return:
    /// - `Ok(true)`  → continue processing (proceed to authentication)
    /// - `Ok(false)` → reject this connection and wait for the next one
    /// - `Err(_)`    → treat as a failure and skip this connection
    fn preprocess_connection(&self, stream: &mut Self::Stream) -> anyhow::Result<bool> {
        let _ = stream;
        Ok(true)
    }
    /// Authenticate incoming connection
    ///
    /// Return `true` to accept connection.
    fn auth(&self, stream: &mut Self::Stream) -> anyhow::Result<bool> {
        let _ = stream;
        Ok(true)
    }

    /// Provide stream upgrade function (e.g., handshake, encryption)
    fn upgrade_stream(&self, stream: Self::Stream) -> anyhow::Result<Self::UpgradeStream>;

    /// Post-process upgraded connection
    ///
    /// Called after stream upgrade (e.g., encryption established).
    fn postprocess_connection(&self, stream: &mut Self::UpgradeStream) -> anyhow::Result<()> {
        let _ = stream;
        Ok(())
    }

    /// Create progress bar
    fn create_progress_bar(&self, total: u64) -> Box<dyn ProgressBar>;

    /// Start UDP broadcaster
    ///
    /// Returns:
    /// - stop function
    /// - thread handle
    fn start_broadcaster(
        &self,
        listener_addr: SocketAddr,
    ) -> (impl FnOnce(), std::thread::JoinHandle<()>);
}

/// Run receiver runtime.
///
/// Handles full lifecycle:
/// - accept connection
/// - receive files
/// - send files
pub fn run_v1_0<A, P, I, F>(
    app: A,
    files_to_send: impl Iterator<Item = P>,
    create_listener: F,
) -> anyhow::Result<()>
where
    A: App,
    P: AsRef<Path>,
    I: Iterator<Item = io::Result<A::Stream>> + Send + 'static,
    F: Fn(&A) -> anyhow::Result<(SocketAddr, I)>,
{
    // Create TCP listener
    let (listen_addr, incoming_streams) = create_listener(&app)?;

    // Start broadcaster (optional)
    let broadcaster = if !app.disable_broadcaster() {
        Some(app.start_broadcaster(listen_addr))
    } else {
        None
    };

    // Accept authenticated connection
    let stream = accept_authenticated_stream(&app, incoming_streams).with_context(|| {
        format!(
            "Failed to accept authenticated connection on {}",
            listen_addr
        )
    })?;

    // Stop broadcaster after connection is established
    if let Some((stop, handle)) = broadcaster {
        stop();
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("Broadcaster thread panicked"))?;
    }

    // Upgrade stream (e.g., encryption)
    let mut stream = app.upgrade_stream(stream)?;

    app.postprocess_connection(&mut stream)
        .context("postprocess faild")?;

    // Receive loop
    loop {
        let mut marker = [0u8; 5];
        stream.read_exact(&mut marker)?;

        match &marker {
            b":fff:" => {
                receiver_receive_file(&app, &mut stream)?;
            }
            b":eof:" => break,
            _ => unreachable!("Invalid protocol marker"),
        }
    }

    // Send files
    for path in files_to_send {
        receiver_send_file(&app, path, &mut stream)?;
    }

    // End session
    stream.write_all(b":eof:")?;
    stream.flush()?;
    Ok(())
}

/// Accept first authenticated stream from incoming connections.
///
/// Iterates over incoming streams and returns the first one
/// that passes authentication.
fn accept_authenticated_stream<A: App, L>(app: &A, incoming: L) -> anyhow::Result<A::Stream>
where
    L: Iterator<Item = io::Result<A::Stream>>,
{
    for stream in incoming {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        match app.preprocess_connection(&mut stream) {
            Ok(false) | Err(_) => continue,
            _ => {}
        }

        match match_bytes("fs-share:v1.0\n", &mut stream) {
            Ok(true) => {
                stream.write_all(b":accept:")?;
                stream.flush()?;
            }
            Ok(false) => {
                let _ = stream.write_all(b":reject:");
                let _ = stream.flush();
                continue;
            }
            Err(_) => continue,
        }

        match app.auth(&mut stream) {
            Ok(true) => return Ok(stream),
            _ => continue,
        }
    }

    anyhow::bail!("No authenticated connection found")
}

fn match_bytes<B: AsRef<[u8]>, R: Read>(bytes: B, mut reader: R) -> anyhow::Result<bool> {
    let expected = bytes.as_ref();

    let mut buf = vec![0u8; expected.len()].into_boxed_slice();

    reader
        .read_exact(&mut buf)
        .context("Failed to read bytes from reader")?;

    Ok(buf.as_ref() == expected)
}
