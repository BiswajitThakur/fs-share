//! # UDP Broadcaster
//!
//! This module provides a UDP-based broadcaster used for service discovery
//! in local networks.
//!
//! It periodically sends structured packets to a broadcast address,
//! allowing receivers to detect available services.
//!
//! ## Use Case
//!
//! Used in `fs-share` to announce receiver availability so that senders
//! can automatically discover peers on the same LAN.
//!
//! ## Packet Format
//!
//! Each broadcast packet:
//!
//! ```text
//! [header][payload]
//! ```
//!
//! Payload consists of multiple fields:
//!
//! ```text
//! :<len:u16><bytes>
//! :<len:u16><bytes>
//! ...
//! ```
//!
//! This matches the format used by [`crate::broadcast::receiver::PayloadReader`] on the receiver side.
//!
use std::{
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    time::Duration,
};

use anyhow::Context;

/// UDP broadcaster.
///
pub struct Broadcaster {
    /// Packet header used to identify valid messages
    header: Vec<u8>,

    /// Encoded payload (structured fields)
    payload: Vec<u8>,

    /// Local address to bind UDP socket
    bind_addr: SocketAddr,

    /// Target broadcast address
    target_addr: SocketAddr,

    /// Optional dynamic interval (milliseconds)
    interval_ms: Option<Arc<AtomicU64>>,
}

impl Broadcaster {
    /// Create a new builder for configuring [`Broadcaster`]
    pub fn builder() -> BroadcasterBuilder {
        BroadcasterBuilder::default()
    }

    /// Start broadcasting in a background thread.
    ///
    /// Returns:
    /// - Stop function
    /// - Thread handle
    ///
    /// ## Behavior
    ///
    /// - Sends packets periodically
    /// - Stops when stop function is called
    /// - Logs errors to stderr
    pub fn start(self) -> (impl FnOnce(), std::thread::JoinHandle<()>) {
        let (stop_tx, stop_rx) = mpsc::channel();

        let handle = std::thread::spawn(move || {
            if let Err(e) = self.run(stop_rx) {
                eprintln!("Broadcaster error: {}", e);
            }
        });

        let stop = move || {
            let _ = stop_tx.send(());
        };

        (stop, handle)
    }

    /// Internal run loop.
    ///
    /// Builds the packet and continuously sends it until stopped.
    fn run(self, stop_rx: Receiver<()>) -> anyhow::Result<()> {
        // Build final packet = header + payload
        let mut packet = Vec::with_capacity(self.header.len() + self.payload.len());
        packet.extend_from_slice(&self.header);
        packet.extend_from_slice(&self.payload);

        let socket = UdpSocket::bind(self.bind_addr)
            .with_context(|| format!("Failed to bind UDP socket on {}", self.bind_addr))?;

        socket
            .set_broadcast(true)
            .context("Failed to enable broadcast on UDP socket")?;

        loop {
            match stop_rx.recv_timeout(self.get_interval()) {
                // Stop signal received
                Ok(_) => break,

                // Timeout → send packet
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    socket.send_to(&packet, self.target_addr).with_context(|| {
                        format!("Failed to send broadcast packet to {}", self.target_addr)
                    })?;
                }

                // Channel disconnected → also stop
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("Stop channel disconnected. Stopping broadcaster.");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get current broadcast interval.
    ///
    /// If dynamic interval is provided, reads from `AtomicU64`,
    /// otherwise defaults to 300 ms.
    fn get_interval(&self) -> Duration {
        Duration::from_millis(
            self.interval_ms
                .as_ref()
                .map(|v| v.load(Ordering::Relaxed))
                .unwrap_or(300),
        )
    }
}

use std::net::{IpAddr, Ipv4Addr};

/// Builder for [`Broadcaster`]
///
/// Provides configuration for:
/// - Header
/// - Payload fields
/// - Bind address
/// - Broadcast target address
/// - Interval
pub struct BroadcasterBuilder {
    header: Vec<u8>,
    payload: Vec<u8>,
    bind_addr: SocketAddr,
    target_addr: SocketAddr,
    interval_ms: Option<Arc<AtomicU64>>,
}

impl Default for BroadcasterBuilder {
    fn default() -> Self {
        Self {
            header: Vec::new(),
            payload: Vec::new(),
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            target_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), 7755),
            interval_ms: None,
        }
    }
}

impl BroadcasterBuilder {
    /// Set packet header
    pub fn header<T: Into<Vec<u8>>>(mut self, header: T) -> Self {
        self.header = header.into();
        self
    }

    /// Add a payload field.
    ///
    /// Encoded as:
    /// ```text
    /// :<len:u16><bytes>
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if field size exceeds `u16::MAX`
    pub fn add_field<T: AsRef<[u8]>>(mut self, data: T) -> Self {
        let bytes = data.as_ref();

        assert!(bytes.len() < u16::MAX as usize, "field too large");

        self.payload.push(b':');
        self.payload
            .extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        self.payload.extend_from_slice(bytes);

        self
    }

    /// Set dynamic broadcast interval (milliseconds)
    pub fn interval(mut self, interval: Arc<AtomicU64>) -> Self {
        self.interval_ms = Some(interval);
        self
    }

    /// Set UDP bind address
    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    /// Set broadcast target address
    pub fn target_addr(mut self, addr: SocketAddr) -> Self {
        self.target_addr = addr;
        self
    }

    /// Build [`Broadcaster`]
    pub fn build(self) -> Broadcaster {
        Broadcaster {
            header: self.header,
            payload: self.payload,
            bind_addr: self.bind_addr,
            target_addr: self.target_addr,
            interval_ms: self.interval_ms,
        }
    }
}
