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

pub struct BroadcastReceiver {
    prefix: Vec<u8>,
    buffer: Box<[u8]>,
    socket: UdpSocket,
}

impl BroadcastReceiver {
    pub fn builder() -> BroadcastReceiverBuilder {
        BroadcastReceiverBuilder::default()
    }
}

pub struct PayloadReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> PayloadReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'a> Iterator for PayloadReader<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.buf.len() {
            return None;
        }

        if self.buf[self.pos] != b':' {
            return None;
        }
        self.pos += 1;

        if self.pos + 2 > self.buf.len() {
            return None;
        }

        let len = u16::from_be_bytes([self.buf[self.pos], self.buf[self.pos + 1]]) as usize;

        self.pos += 2;

        if self.pos + len > self.buf.len() {
            return None;
        }

        let slice = &self.buf[self.pos..self.pos + len];
        self.pos += len;

        Some(slice)
    }
}

impl BroadcastReceiver {
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
            let mut seen: HashMap<SocketAddr, U> = HashMap::new();

            loop {
                if stop_rx.try_recv().is_ok() {
                    // println!("Receiver stopped.");
                    break;
                }

                match this.socket.recv_from(&mut this.buffer) {
                    Ok((size, addr)) => {
                        if this.buffer.starts_with(&this.prefix) {
                            let payload = &this.buffer[this.prefix.len()..size];
                            let reader = PayloadReader::new(payload);

                            match U::try_from((addr, reader)) {
                                Ok(data) => {
                                    let is_new_or_changed = match seen.get(&addr) {
                                        Some(old) => old != &data,
                                        None => true,
                                    };

                                    if is_new_or_changed {
                                        seen.insert(addr, data.clone());
                                        let _ = data_tx.send((addr, data));
                                    }
                                }
                                Err(_) => {
                                    // silently ignore invalid payload
                                    continue;
                                }
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
            buffer_size: NonZero::new(8 * 1024),
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 7755),
        }
    }
}

impl BroadcastReceiverBuilder {
    pub fn prefix<T: Into<Vec<u8>>>(mut self, value: T) -> Self {
        self.prefix = value.into();
        self
    }

    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = NonZero::new(size);
        self
    }

    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

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

        let receiver = BroadcastReceiver {
            prefix: self.prefix,
            buffer,
            socket,
        };

        Ok(receiver)
    }
}
