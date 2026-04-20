use std::{
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    time::Duration,
};

pub struct Broadcaster {
    header: Vec<u8>,
    payload: Vec<u8>,
    bind_addr: SocketAddr,
    target_addr: SocketAddr,
    interval_ms: Option<Arc<AtomicU64>>,
}

impl Broadcaster {
    pub fn builder() -> BroadcasterBuilder {
        BroadcasterBuilder::default()
    }

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
    fn run(self, stop_rx: Receiver<()>) -> anyhow::Result<()> {
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
                Ok(_) => {
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    socket.send_to(&packet, self.target_addr).with_context(|| {
                        format!("Failed to send broadcast packet to {}", self.target_addr)
                    })?;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // sender dropped → treat as stop signal
                    println!("Stop channel disconnected. Stopping broadcaster.");
                    break;
                }
            }
        }

        Ok(())
    }
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

use anyhow::Context;

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
    pub fn header<T: Into<Vec<u8>>>(mut self, header: T) -> Self {
        self.header = header.into();
        self
    }

    pub fn add_field<T: AsRef<[u8]>>(mut self, data: T) -> Self {
        let bytes = data.as_ref();

        assert!(bytes.len() < u16::MAX as usize, "field too large");

        self.payload.push(b':');
        self.payload
            .extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        self.payload.extend_from_slice(bytes);

        self
    }

    pub fn interval(mut self, interval: Arc<AtomicU64>) -> Self {
        self.interval_ms = Some(interval);
        self
    }

    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    pub fn target_addr(mut self, addr: SocketAddr) -> Self {
        self.target_addr = addr;
        self
    }

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
