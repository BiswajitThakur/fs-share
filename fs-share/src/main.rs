use std::{
    io::{self, stdout, BufReader, BufWriter},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    thread,
};

use indicatif::MultiProgress;
use share_utils::{ClientType, RecvDataWithoutPd, RecvType, SendData, TransmissionMode};

mod cli;

const BROADCAST_PORT: u16 = 34345;

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    let multi_pd = MultiProgress::new();
    let option = std::env::args().skip(1).next().unwrap();
    match option.as_str() {
        "send" | "--send" => {
            let client = ClientType::sender()
                .set_broadcast_addr(SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
                    BROADCAST_PORT,
                ))
                .build();
            let sender = client.connect(&mut stdout, None)?;
            handle_connection_sender_without_pd(&mut stdout, sender)?;
        }
        "receive" | "--receive" => {
            let client = ClientType::receiver()
                .set_tcp_listener_addr("[::]:0".parse().unwrap())
                .set_udp_socket_addr("[::]:0".parse().unwrap())
                .set_broadcast_addr(SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
                    BROADCAST_PORT,
                ))
                .build();
            let receiver = client.connect(&mut stdout, Some("eagle1234"))?;
            handle_connection_receiver(&mut stdout, receiver)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_connection_sender_without_pd<W: io::Write>(
    stdout: &mut W,
    mode: TransmissionMode<TcpStream>,
) -> io::Result<()> {
    match mode {
        TransmissionMode::HalfDuplex(stream) => {
            handle_half_duplex_sender_without_pd(stdout, stream)
        }
        TransmissionMode::FullDuplex(stream1, stream2) => {
            handle_full_duplex_without_pd(stdout, stream1, stream2)
        }
    }
}
fn handle_connection_receiver<W: io::Write>(
    stdout: &mut W,
    mode: TransmissionMode<TcpStream>,
) -> io::Result<()> {
    match mode {
        TransmissionMode::HalfDuplex(stream) => handle_half_duplex_receiver(stdout, stream),
        TransmissionMode::FullDuplex(stream1, stream2) => {
            handle_full_duplex_without_pd(stdout, stream1, stream2)
        }
    }
}

fn handle_half_duplex_sender_without_pd<W: io::Write>(
    stdout: &mut W,
    mut stream: TcpStream,
) -> io::Result<()> {
    {
        let mut sender = BufWriter::new(&mut stream);
        for file in std::env::args().skip(2) {
            SendData::File(file).send_without_progress(stdout, &mut sender)?;
        }
        SendData::<PathBuf>::Eof.send_without_progress(stdout, &mut sender)?;
    }
    let mut receiver = BufReader::new(stream);
    loop {
        match receiver.recv_data_without_pd(stdout) {
            Ok(RecvType::Eof) => break,
            Ok(RecvType::File(name)) => {
                println!("File Received: {}", name);
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn handle_half_duplex_receiver<W: io::Write>(
    stdout: &mut W,
    mut stream: TcpStream,
) -> io::Result<()> {
    let mut receiver = BufReader::new(&mut stream);
    loop {
        match receiver.recv_data_without_pd(stdout) {
            Ok(RecvType::Eof) => break,
            Ok(RecvType::File(name)) => {
                println!("File Received: {}", name);
            }
            Err(err) => return Err(err),
        }
    }
    let mut sender = BufWriter::new(&mut stream);
    for file in std::env::args().skip(2) {
        SendData::File(file).send_without_progress(stdout, &mut sender)?;
    }
    SendData::<PathBuf>::Eof.send_without_progress(stdout, &mut sender)?;
    Ok(())
}

fn handle_full_duplex_without_pd<W: io::Write>(
    stdout: &mut W,
    stream1: TcpStream,
    stream2: TcpStream,
) -> io::Result<()> {
    let mut receiver = BufReader::new(stream1);
    let sender = BufWriter::new(stream2);
    let files = std::env::args().skip(2).collect();
    let handler =
        thread::spawn(move || send_files_without_progress(&mut io::stdout(), sender, files));
    loop {
        match receiver.recv_data_without_pd(stdout) {
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

fn send_files_without_progress<U: io::Write, W: io::Write, T: AsRef<Path>>(
    stdout: &mut U,
    mut stream: BufWriter<W>,
    files: Vec<T>,
) -> io::Result<()> {
    for file in files {
        SendData::File(file).send_without_progress(stdout, &mut stream)?;
    }
    SendData::<T>::Eof.send_without_progress(stdout, &mut stream)
}
