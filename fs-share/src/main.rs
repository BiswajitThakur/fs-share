use std::{
    io,
    net::{IpAddr, Ipv6Addr, SocketAddr, TcpStream},
};

use share_utils::{ClientType, TransmissionMode};

mod cli;

const BROADCAST_PORT: u16 = 34345;

fn main() -> std::io::Result<()> {
    let option = std::env::args().skip(1).next().unwrap();
    match option.as_str() {
        "send" | "--send" => {
            let client = ClientType::sender()
                .set_broadcast_addr(SocketAddr::new(
                    IpAddr::V6(Ipv6Addr::LOCALHOST),
                    BROADCAST_PORT,
                ))
                .build();
            let mode = client.connect()?;
            handle_sender(mode)?;
        }
        "receive" | "--receive" => {
            let client = ClientType::receiver()
                .set_tcp_listener_addr(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0))
                .set_udp_socket_addr(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0))
                .set_broadcast_addr(SocketAddr::new(
                    IpAddr::V6(Ipv6Addr::LOCALHOST),
                    BROADCAST_PORT,
                ))
                .build();
            let mode = client.connect()?;
            handle_receiver(mode)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_sender(mode: TransmissionMode<TcpStream>) -> io::Result<()> {
    match mode {
        TransmissionMode::HalfDuplex(stream) => handle_half_duplex_sender(stream),
        TransmissionMode::FullDuplex(stream1, stream2) => {
            handle_full_duplex_sender(stream1, stream2)
        }
    }
}

fn handle_half_duplex_sender(_stream: TcpStream) -> io::Result<()> {
    todo!()
}

fn handle_full_duplex_sender(_stream1: TcpStream, _stream2: TcpStream) -> io::Result<()> {
    todo!()
}

fn handle_receiver(mode: TransmissionMode<TcpStream>) -> io::Result<()> {
    match mode {
        TransmissionMode::HalfDuplex(stream) => handle_half_duplex_receiver(stream),
        TransmissionMode::FullDuplex(stream1, stream2) => {
            handle_full_duplex_receiver(stream1, stream2)
        }
    }
}

fn handle_half_duplex_receiver(_stream: TcpStream) -> io::Result<()> {
    todo!()
}

fn handle_full_duplex_receiver(_stream1: TcpStream, _stream2: TcpStream) -> io::Result<()> {
    todo!()
}
