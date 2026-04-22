use std::{
    borrow::Cow,
    fmt::Display,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    thread::JoinHandle,
    time::Duration,
};

use anyhow::Context;
use colored::Colorize;
use fs_share_utils::{
    broadcast::receiver::PayloadReader,
    pb::ProgressBar,
    sender::{App, ReceiverData as RD},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ReceiverData {
    name: String,
    os: String,
    arch: String,
    addr: SocketAddr,
}

impl Display for ReceiverData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name: {}, ", self.name)?;
        write!(f, "OS: {} ({}), ", self.os, self.arch)?;
        write!(f, "Addr: {}", self.addr)?;
        Ok(())
    }
}

impl<'a> TryFrom<(SocketAddr, PayloadReader<'a>)> for ReceiverData {
    type Error = ();
    fn try_from(value: (SocketAddr, PayloadReader)) -> Result<Self, Self::Error> {
        let a = value.0;
        let mut value = value.1;
        let v = value.next();
        if v.is_none() {
            return Err(());
        }
        let name = unsafe { str::from_utf8_unchecked(v.unwrap()) };

        let v = value.next();
        if v.is_none() {
            return Err(());
        }
        let os = unsafe { str::from_utf8_unchecked(v.unwrap()) };

        let v = value.next();
        if v.is_none() {
            return Err(());
        }
        let arch = unsafe { str::from_utf8_unchecked(v.unwrap()) };

        let v = value.next();
        if v.is_none() {
            return Err(());
        }
        let addr = unsafe { str::from_utf8_unchecked(v.unwrap()) };
        let addr = addr.parse::<SocketAddr>();
        if addr.is_err() {
            return Err(());
        }
        let mut addr = addr.unwrap();
        if matches!(
            addr.ip(),
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
                | IpAddr::V4(Ipv4Addr::LOCALHOST)
                | IpAddr::V6(Ipv6Addr::UNSPECIFIED)
                | IpAddr::V6(Ipv6Addr::LOCALHOST)
        ) {
            addr = SocketAddr::new(a.ip(), addr.port());
        }

        Ok(Self {
            name: name.to_owned(),
            os: os.to_owned(),
            arch: arch.to_owned(),
            addr,
        })
    }
}

impl RD for ReceiverData {
    fn addr(&self) -> SocketAddr {
        self.addr
    }
}

pub struct SenderAppV1<U> {
    pub broadcast_addr: SocketAddr,
    pub receiver_addr: Option<SocketAddr>,
    pub download_dir: PathBuf,
    pub upgrade_stream: Box<dyn Fn(TcpStream) -> anyhow::Result<U> + 'static>,
    pub pb: Box<dyn Fn(u64) -> Box<dyn ProgressBar>>,
}

impl<U: Read + Write> App for SenderAppV1<U> {
    type Stream = TcpStream;
    type UpgradeStream = U;
    fn prefix(&self) -> &str {
        "v1.fs-share"
    }
    fn broadcast_addr(&self) -> SocketAddr {
        self.broadcast_addr
    }
    fn receiver_addr(&self) -> Option<SocketAddr> {
        self.receiver_addr
    }
    fn download_dir<'a>(&'a self) -> Cow<'a, Path> {
        Cow::Borrowed(&self.download_dir)
    }
    fn upgrade_stream(&self, stream: Self::Stream) -> anyhow::Result<Self::UpgradeStream> {
        (*self.upgrade_stream)(stream)
    }
    fn create_progress_bar(&self, n: u64) -> Box<dyn ProgressBar> {
        (self.pb)(n)
    }
    fn preprocess_connection(&self, stream: &mut Self::Stream) -> anyhow::Result<()> {
        let addr = stream.local_addr()?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(100)))
            .with_context(|| format!("Faild to set read timeout on {}", addr))?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_millis(100)))
            .with_context(|| format!("Faild to set write timeout on {}", addr))?;
        Ok(())
    }

    fn select_receiver_addr<V>(
        &self,
        (stop, rx, handle): (
            Box<dyn FnOnce() + Send>,
            Receiver<(SocketAddr, V)>,
            JoinHandle<()>,
        ),
    ) -> Option<SocketAddr>
    where
        V: Clone + std::fmt::Display + PartialEq + RD + Send + 'static,
    {
        use std::sync::mpsc;

        let mut items: Vec<V> = Vec::new();

        println!(
            "{}",
            "Searching for receivers... (press ENTER to stop)".blue()
        );

        let (input_tx, input_rx) = mpsc::channel();
        let t = std::thread::spawn(move || {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            let _ = input_tx.send(());
        });

        loop {
            // check ENTER pressed
            if input_rx.try_recv().is_ok() {
                break;
            }

            // receive network data
            match rx.try_recv() {
                Ok(data) => {
                    if !items.contains(&data.1) {
                        println!(
                            "[{}] {}",
                            (items.len() + 1).to_string().blue(),
                            data.1.to_string().green()
                        );
                        items.push(data.1);
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }

        // stop background receiver
        stop();
        let _ = handle.join();
        let _ = t.join();

        if items.is_empty() {
            println!("No receivers found.");
            return None;
        }

        print!("-----------------\nSelect receiver index: ");
        let _ = std::io::stdout().flush();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;

        let idx: usize = input.trim().parse().ok()?;

        items.get(idx - 1).map(|v| v.addr())
    }
}
