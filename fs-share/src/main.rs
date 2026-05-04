use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};

use anyhow::Context;
use clap::Parser;
use fs_share_utils::{receiver::run_v1_0 as run_receiver_app, sender::run_v1_0 as run_sender_app};
use socket2::{Domain, Socket, Type};

use crate::{
    cli::Mode,
    pb::{my_pb, no_pb},
    receiver::ReceiverApp,
    sender::{ReceiverData, SenderAppV1},
    utils::{create_tcp_listener, receiver_upgrade_stream, select_ip, sender_upgrade_stream},
};

mod cli;
mod pb;
mod receiver;
mod sender;
mod utils;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    match cli.mode {
        Mode::Send {
            receiver_addr,
            download_dir,
            disable_progress,
            broadcast_port,
            args,
        } => {
            let mut app = SenderAppV1 {
                broadcast_addr: SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::UNSPECIFIED,
                    broadcast_port,
                )),
                receiver_addr,
                download_dir: download_dir.unwrap_or("./".into()),
                upgrade_stream: Box::new(sender_upgrade_stream),
                pb: Box::new(my_pb),
            };
            if disable_progress {
                app.pb = Box::new(no_pb);
            }

            //run_sender_app::<_, _, _, ReceiverData>(app, args.iter(), TcpStream::connect)?;
            run_sender_app::<_, _, _, ReceiverData>(app, args.iter(), |addr| {
                let domain = if addr.is_ipv6() {
                    Domain::IPV6
                } else {
                    Domain::IPV4
                };
                let socket = Socket::new(domain, Type::STREAM, None)?;
                socket.set_recv_buffer_size(256 * 1024)?;
                socket.set_send_buffer_size(256 * 1024)?;
                socket.connect(&addr.into())?;
                let stream = TcpStream::from(socket);
                stream.set_nodelay(true)?;
                Ok(stream)
            })?;
        }
        Mode::Receive {
            tcp_listener_addr,
            download_dir,
            disable_broadcast,
            disable_progress,
            broadcast_port,
            args,
        } => {
            let addr = match tcp_listener_addr {
                Some(v) => v,
                None => {
                    let ip = select_ip().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
                    SocketAddr::new(ip, 0)
                }
            };
            let mut app = ReceiverApp {
                broadcast_addr: SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::BROADCAST,
                    broadcast_port,
                )),
                download_dir: download_dir.unwrap_or("./".into()),
                disable_broadcaster: disable_broadcast,
                upgrade_stream: Box::new(receiver_upgrade_stream),
                pb: Box::new(my_pb),
            };

            if disable_progress {
                app.pb = Box::new(no_pb);
            }
            run_receiver_app(app, args.iter(), |_| create_tcp_listener(addr))?;
        }
    }
    Ok(())
}
