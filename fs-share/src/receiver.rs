use std::{
    borrow::Cow,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
    path::{Path, PathBuf},
};

use anyhow::Context;
use fs_share_utils::{broadcast::sender::Broadcaster, pb::ProgressBar, receiver::App};

pub struct ReceiverApp<U> {
    pub broadcast_addr: SocketAddr,
    pub download_dir: PathBuf,
    pub disable_broadcaster: bool,
    pub upgrade_stream: Box<dyn Fn(TcpStream) -> anyhow::Result<U> + 'static>,
    pub pb: Box<dyn Fn(u64) -> Box<dyn ProgressBar>>,
}

impl<U: Read + Write> App for ReceiverApp<U> {
    type Stream = TcpStream;
    type UpgradeStream = U;
    fn prefix(&self) -> &str {
        "v1.fs-share"
    }
    fn broadcast_addr(&self) -> SocketAddr {
        self.broadcast_addr
    }
    fn download_dir<'a>(&'a self) -> Cow<'a, Path> {
        Cow::Borrowed(&self.download_dir)
    }
    fn disable_broadcaster(&self) -> bool {
        self.disable_broadcaster
    }
    fn upgrade_stream(&self, stream: Self::Stream) -> anyhow::Result<Self::UpgradeStream> {
        (*self.upgrade_stream)(stream)
    }
    fn preprocess_connection(&self, stream: &mut Self::Stream) -> anyhow::Result<bool> {
        let addr = stream.local_addr()?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(100)))
            .with_context(|| format!("Faild to set read timeout on {}", addr))?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_millis(100)))
            .with_context(|| format!("Faild to set write timeout on {}", addr))?;
        Ok(true)
    }
    fn create_progress_bar(&self, n: u64) -> Box<dyn ProgressBar> {
        (self.pb)(n)
    }
    fn start_broadcaster(
        &self,
        listener_addr: SocketAddr,
    ) -> (impl FnOnce(), std::thread::JoinHandle<()>) {
        let bc_sender = Broadcaster::builder()
            .header(self.prefix())
            .target_addr(self.broadcast_addr())
            .bind_addr(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))
            .add_field(
                ["USER", "USERNAME"]
                    .iter()
                    .find_map(|&key| std::env::var(key).ok())
                    .unwrap_or("Unknown".into()),
            )
            .add_field(std::env::consts::OS)
            .add_field(std::env::consts::ARCH)
            .add_field(listener_addr.to_string())
            .build();

        bc_sender.start()
    }
}
