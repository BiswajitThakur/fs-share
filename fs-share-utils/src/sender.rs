use std::{
    borrow::Cow,
    fmt::Display,
    io::{self, Read, Write},
    net::SocketAddr,
    path::Path,
    sync::mpsc::Receiver,
    thread::JoinHandle,
};

use anyhow::Context;

use crate::{
    broadcast::receiver::{BroadcastReceiver, PayloadReader},
    pb::ProgressBar,
    tf::{sender_receive_file, sender_send_file},
};

pub trait ReceiverData {
    fn addr(&self) -> SocketAddr;
}
pub trait App {
    type Stream: Read + Write;
    type UpgradeStream: Read + Write;

    fn prefix(&self) -> &str;
    fn broadcast_addr(&self) -> SocketAddr;
    fn receiver_addr(&self) -> Option<SocketAddr>;
    fn download_dir<'a>(&'a self) -> Cow<'a, Path>;
    fn auth(&self, stream: &mut Self::Stream) -> io::Result<bool> {
        let _ = stream;
        Ok(true)
    }
    fn get_upgrade_stream(&self) -> impl Fn(Self::Stream) -> anyhow::Result<Self::UpgradeStream>;
    fn create_pb(&self, n: u64) -> Box<dyn ProgressBar>;
    fn select_receiver_addr<U>(
        &self,
        v: (
            Box<dyn FnOnce() + Send>,
            Receiver<(SocketAddr, U)>,
            JoinHandle<()>,
        ),
    ) -> Option<SocketAddr>
    where
        U: Clone + Display + PartialEq,
        U: ReceiverData + Send + 'static;
}

pub fn run<A, P, F, R>(app: A, files: impl Iterator<Item = P>, f: F) -> anyhow::Result<()>
where
    A: App,
    P: AsRef<Path>,
    F: Fn(&A, SocketAddr) -> anyhow::Result<A::Stream>,
    R: for<'a> TryFrom<(SocketAddr, PayloadReader<'a>)>
        + ReceiverData
        + Clone
        + Display
        + PartialEq
        + Send
        + 'static,
{
    let v = match app.receiver_addr() {
        a @ Some(_) => a,
        None => {
            let bc = BroadcastReceiver::builder()
                .prefix(app.prefix())
                .bind_addr(app.broadcast_addr())
                .buffer_size(4 * 1024)
                .build()
                .context("Failed to build BroadcastReceiver")?;
            let v = bc.start::<R>();
            app.select_receiver_addr(v)
        }
    };

    let addr = v.context("No valid server address received from broadcast")?;

    let stream = f(&app, addr)?;
    let mut stream = app.get_upgrade_stream()(stream)?;
    for path in files {
        sender_send_file(&app, path, &mut stream)?;
    }
    stream.write_all(b":eof:")?;
    stream.flush()?;
    loop {
        let mut magic = [0u8; 5];
        stream.read_exact(&mut magic)?;
        match &magic {
            b":fff:" => {
                sender_receive_file(&app, &mut stream)?;
            }
            b":eof:" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}
