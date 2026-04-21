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

    /// Authenticate incoming connection
    ///
    /// Return `true` to accept connection.
    fn auth(&self, stream: &mut Self::Stream) -> io::Result<bool> {
        let _ = stream;
        Ok(true)
    }

    /// Provide stream upgrade function (e.g., handshake, encryption)
    fn upgrade_stream(&self) -> impl Fn(Self::Stream) -> anyhow::Result<Self::UpgradeStream>;

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
pub fn run_v1<A, P, I, F>(
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
    let mut stream =
        accept_authenticated_stream(incoming_streams, |s| app.auth(s)).with_context(|| {
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

    /*
    if !match_bytes("fs-share-v1.0.0\n", &mut stream)? {
        anyhow::bail!("Version Not Match");
    } else {
        stream.write_all(b":accept:")?;
        stream.flush()?;
        //stream.write_all(b":reject:");
    };
    */

    // Upgrade stream (e.g., encryption)
    let mut stream = app.upgrade_stream()(stream)?;

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

const VERSION: &str = "";

/// Accept first authenticated stream from incoming connections.
///
/// Iterates over incoming streams and returns the first one
/// that passes authentication.
fn accept_authenticated_stream<T, L>(
    incoming: L,
    auth: impl Fn(&mut T) -> io::Result<bool>,
) -> io::Result<T>
where
    T: Read + Write,
    L: Iterator<Item = io::Result<T>>,
{
    for stream in incoming {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        match auth(&mut stream) {
            Ok(true) => return Ok(stream),
            _ => continue,
        }
    }

    Err(io::Error::other("No authenticated connection found"))
}

fn match_bytes<B: AsRef<[u8]>, R: Read>(bytes: B, mut reader: R) -> anyhow::Result<bool> {
    let expected = bytes.as_ref();

    let mut buf = vec![0u8; expected.len()].into_boxed_slice();

    reader
        .read_exact(&mut buf)
        .context("Failed to read bytes from reader")?;

    Ok(buf.as_ref() == expected)
}
