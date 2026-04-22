//! # UDP Broadcast Receiver
//!
//! This module provides a lightweight UDP-based broadcast receiver
//! used for service discovery in local networks.
//!
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    net::{SocketAddr, UdpSocket},
    num::NonZero,
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Context;

/// UDP broadcast receiver.
///
/// Listens for UDP packets, filters them using a prefix,
/// and emits parsed data via a channel.
pub struct BroadcastReceiver {
    /// Packet prefix used to identify valid messages
    prefix: Vec<u8>,

    /// Internal buffer used for receiving packets
    buffer: Box<[u8]>,

    /// UDP socket bound to a local address
    socket: UdpSocket,
}

impl BroadcastReceiver {
    /// Create a new builder for configuring [`BroadcastReceiver`]
    pub fn builder() -> BroadcastReceiverBuilder {
        BroadcastReceiverBuilder::default()
    }
}

/// Iterator over structured payload data.
///
/// Payload format:
/// ```text
/// :<len:u16><bytes>
/// :<len:u16><bytes>
/// ...
/// ```
///
/// Each field is prefixed with `:` and a 2-byte big-endian length.
pub struct PayloadReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> PayloadReader<'a> {
    /// Create a new payload reader from raw bytes
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'a> Iterator for PayloadReader<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        // End of buffer
        if self.pos >= self.buf.len() {
            return None;
        }

        // Expect field marker ':'
        unsafe {
            if *self.buf.get_unchecked(self.pos) != b':' {
                return None;
            }
        }
        self.pos += 1;

        // Ensure enough bytes for length
        if self.pos + 2 > self.buf.len() {
            return None;
        }

        let len = u16::from_be_bytes([self.buf[self.pos], self.buf[self.pos + 1]]) as usize;

        self.pos += 2;

        // Ensure enough bytes for data
        if self.pos + len > self.buf.len() {
            return None;
        }

        let slice = &self.buf[self.pos..self.pos + len];
        self.pos += len;

        Some(slice)
    }
}

impl BroadcastReceiver {
    /// Start receiving broadcast packets in a background thread.
    ///
    /// Returns:
    /// - Stop function
    /// - Data receiver channel
    /// - Thread handle
    ///
    /// ## Type Parameter
    ///
    /// `U` is a user-defined type that can be constructed from:
    /// ```text
    /// (SocketAddr, PayloadReader)
    /// ```
    ///
    /// This allows flexible decoding of incoming packets.
    ///
    /// ## Behavior
    ///
    /// - Deduplicates data per sender (`SocketAddr`)
    /// - Sends only new or changed data
    /// - Ignores invalid payloads silently
    pub fn start<U>(
        self,
    ) -> (
        Box<dyn FnOnce() + Send>,
        Receiver<(SocketAddr, U)>,
        JoinHandle<()>,
    )
    where
        U: for<'a> TryFrom<(SocketAddr, PayloadReader<'a>)>,
        U: Clone + PartialEq + Send + 'static,
    {
        let (data_tx, data_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut this = self;

            // Track last seen data per sender
            let mut seen: HashMap<SocketAddr, U> = HashMap::new();

            loop {
                // Check stop signal
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                match this.socket.recv_from(&mut this.buffer) {
                    Ok((size, addr)) => {
                        // Validate prefix
                        if this.buffer.starts_with(&this.prefix) {
                            let payload = &this.buffer[this.prefix.len()..size];
                            let reader = PayloadReader::new(payload);

                            match U::try_from((addr, reader)) {
                                Ok(data) => {
                                    // Deduplicate
                                    let is_new_or_changed = match seen.get(&addr) {
                                        Some(old) => old != &data,
                                        None => true,
                                    };

                                    if is_new_or_changed {
                                        seen.insert(addr, data.clone());
                                        let _ = data_tx.send((addr, data));
                                    }
                                }
                                Err(_) => continue, // Ignore invalid payload
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(e) => {
                        eprintln!("Receive error: {}", e);
                        break;
                    }
                }
            }
        });

        let stop = Box::new(move || {
            let _ = stop_tx.send(());
        });

        (stop, data_rx, handle)
    }
}

/// Builder for [`BroadcastReceiver`]
///
pub struct BroadcastReceiverBuilder {
    prefix: Vec<u8>,
    timeout: Option<Duration>,
    buffer_size: Option<NonZero<usize>>,
    bind_addr: SocketAddr,
}

impl Default for BroadcastReceiverBuilder {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            timeout: Some(Duration::from_millis(300)),
            buffer_size: NonZero::new(8 * 1024), // 8 KB
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 7755),
        }
    }
}

impl BroadcastReceiverBuilder {
    /// Set packet prefix for filtering
    pub fn prefix<T: Into<Vec<u8>>>(mut self, value: T) -> Self {
        self.prefix = value.into();
        self
    }

    /// Set internal buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = NonZero::new(size);
        self
    }

    /// Set socket bind address
    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    /// Build [`BroadcastReceiver`]
    pub fn build(self) -> anyhow::Result<BroadcastReceiver> {
        let buffer_size = self.buffer_size.context("Buffer size is not set")?.get();

        let buffer = vec![0u8; buffer_size + self.prefix.len()].into_boxed_slice();

        let socket = UdpSocket::bind(self.bind_addr)
            .with_context(|| format!("Failed to bind UDP socket on {}", self.bind_addr))?;

        socket.set_read_timeout(self.timeout).with_context(|| {
            format!(
                "Failed to set read timeout {:?} on {}",
                self.timeout, self.bind_addr
            )
        })?;

        Ok(BroadcastReceiver {
            prefix: self.prefix,
            buffer,
            socket,
        })
    }
}
