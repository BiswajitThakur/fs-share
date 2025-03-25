use std::{
    io::{self, BufReader, BufWriter},
    net::{IpAddr, Ipv6Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    thread,
};

use share_utils::{ClientType, RecvDataWithoutPd, RecvType, SendData, TransmissionMode};

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
            handle_connection_sender_without_pd(mode)?;
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
            handle_connection_receiver(mode)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_connection_sender_without_pd(mode: TransmissionMode<TcpStream>) -> io::Result<()> {
    match mode {
        TransmissionMode::HalfDuplex(stream) => handle_half_duplex_sender_without_pd(stream),
        TransmissionMode::FullDuplex(stream1, stream2) => {
            handle_full_duplex_without_pd(stream1, stream2)
        }
    }
}
fn handle_connection_receiver(mode: TransmissionMode<TcpStream>) -> io::Result<()> {
    match mode {
        TransmissionMode::HalfDuplex(stream) => handle_half_duplex_receiver(stream),
        TransmissionMode::FullDuplex(stream1, stream2) => {
            handle_full_duplex_without_pd(stream1, stream2)
        }
    }
}

fn handle_half_duplex_sender_without_pd(mut stream: TcpStream) -> io::Result<()> {
    {
        let mut sender = BufWriter::new(&mut stream);
        for file in std::env::args().skip(2) {
            SendData::File(file).send_without_progress(&mut sender)?;
        }
        SendData::<PathBuf>::Eof.send_without_progress(&mut sender)?;
    }
    let mut receiver = BufReader::new(stream);
    loop {
        match receiver.recv_data_without_pd() {
            Ok(RecvType::Eof) => break,
            Ok(RecvType::File(name)) => {
                println!("File Received: {}", name);
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn handle_half_duplex_receiver(mut stream: TcpStream) -> io::Result<()> {
    let mut receiver = BufReader::new(&mut stream);
    loop {
        match receiver.recv_data_without_pd() {
            Ok(RecvType::Eof) => break,
            Ok(RecvType::File(name)) => {
                println!("File Received: {}", name);
            }
            Err(err) => return Err(err),
        }
    }
    let mut sender = BufWriter::new(&mut stream);
    for file in std::env::args().skip(2) {
        SendData::File(file).send_without_progress(&mut sender)?;
    }
    SendData::<PathBuf>::Eof.send_without_progress(&mut sender)?;
    Ok(())
}

fn handle_full_duplex_without_pd(stream1: TcpStream, stream2: TcpStream) -> io::Result<()> {
    let mut receiver = BufReader::new(stream1);
    let sender = BufWriter::new(stream2);
    let files = std::env::args().skip(2).collect();
    let handler = thread::spawn(move || send_files_without_progress(sender, files));
    loop {
        match receiver.recv_data_without_pd() {
            Ok(RecvType::Eof) => break,
            Ok(RecvType::File(name)) => {
                println!("File Received: {}", name);
            }
            Err(err) => return Err(err),
        }
    }
    if let Err(err) = handler.join().unwrap() {
        eprintln!("{}", err);
    };
    Ok(())
}

fn send_files_without_progress<W: io::Write, T: AsRef<Path>>(
    mut stream: BufWriter<W>,
    files: Vec<T>,
) -> io::Result<()> {
    for file in files {
        SendData::File(file).send_without_progress(&mut stream)?;
    }
    SendData::<T>::Eof.send_without_progress(&mut stream)
}
