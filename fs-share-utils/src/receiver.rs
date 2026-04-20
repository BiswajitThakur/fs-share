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

pub trait App {
    type Stream: Read + Write;
    type UpgradeStream: Read + Write;

    fn prefix(&self) -> &str;
    fn broadcast_addr(&self) -> SocketAddr;
    fn download_dir<'a>(&'a self) -> Cow<'a, Path>;
    fn disable_broadcaster(&self) -> bool {
        true
    }

    fn auth(&self, stream: &mut Self::Stream) -> io::Result<bool> {
        let _ = stream;
        Ok(true)
    }

    fn get_upgrade_stream(&self) -> impl Fn(Self::Stream) -> anyhow::Result<Self::UpgradeStream>;
    fn create_pb(&self, n: u64) -> Box<dyn ProgressBar>;

    fn start_broadcaster(
        &self,
        listener_addr: SocketAddr,
    ) -> (impl FnOnce(), std::thread::JoinHandle<()>);
}

pub fn run<A, P, I, F>(app: A, files: impl Iterator<Item = P>, f: F) -> anyhow::Result<()>
where
    A: App,
    P: AsRef<Path>,
    I: Iterator<Item = io::Result<A::Stream>> + Send + 'static,
    F: Fn(&A) -> anyhow::Result<(SocketAddr, I)>,
{
    let (listener_addr, listener) = f(&app)?;

    let broadcaster = if !app.disable_broadcaster() {
        Some(app.start_broadcaster(listener_addr))
    } else {
        None
    };

    let stream = create_stream(listener, |v| app.auth(v))
        .with_context(|| format!("Failed to create authenticated stream on {}", listener_addr))?;

    // stop broadcaster
    if let Some((stop, handler)) = broadcaster {
        stop();
        handler
            .join()
            .map_err(|_| anyhow::anyhow!("Broadcaster thread panicked"))?;
    }
    let mut stream = app.get_upgrade_stream()(stream)?;
    loop {
        let mut magic = [0u8; 5];
        stream.read_exact(&mut magic)?;
        match &magic {
            b":fff:" => {
                receiver_receive_file(&app, &mut stream)?;
            }
            b":eof:" => break,
            _ => unreachable!(),
        }
    }
    for path in files {
        receiver_send_file(&app, path, &mut stream)?;
    }
    stream.write_all(b":eof:")?;
    stream.flush()?;

    Ok(())
}

fn create_stream<T, L>(listener: L, auth: impl Fn(&mut T) -> io::Result<bool>) -> io::Result<T>
where
    T: Read + Write,
    L: Iterator<Item = io::Result<T>>,
{
    for stream in listener {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        match auth(&mut stream) {
            Ok(true) => return Ok(stream),
            _ => continue,
        }
    }

    // This is technically unreachable, but required by type system
    Err(io::Error::other("No valid connection found"))
}
