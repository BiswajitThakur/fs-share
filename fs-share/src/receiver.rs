use std::{
    borrow::Cow,
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
};

use fs_share_utils::{broadcast::sender::Broadcaster, pb::ProgressBar, receiver::App};

pub struct ReceiverAppV1<T, U> {
    pub broadcast_addr: SocketAddr,
    pub download_dir: PathBuf,
    pub disable_broadcaster: bool,
    pub upgrade_stream: Box<dyn Fn(T) -> anyhow::Result<U> + 'static>,
    pub pb: Box<dyn Fn(u64) -> Box<dyn ProgressBar>>,
}

impl<T: Read + Write, U: Read + Write> App for ReceiverAppV1<T, U> {
    type Stream = T;
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
    fn get_upgrade_stream(&self) -> impl Fn(Self::Stream) -> anyhow::Result<Self::UpgradeStream> {
        &*self.upgrade_stream
    }
    fn create_pb(&self, n: u64) -> Box<dyn ProgressBar> {
        (self.pb)(n)
    }
    fn auth(&self, _stream: &mut Self::Stream) -> io::Result<bool> {
        Ok(true)
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
