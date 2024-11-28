use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    thread,
    time::Duration,
};

use clap::{Parser, ValueEnum};
use share_utils::{get_receiver_addr, get_sender_addr, ShareFs};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,

    /// Name
    #[arg(long, default_value_t = String::from("Unknown"))]
    name: String,

    /// password
    #[arg(long, default_value_t = String::from("password"))]
    password: String,
    #[arg(long, default_value_t = 34254)]

    /// port
    port: u16,
    #[arg(long, default_value_t = 60)]

    /// Timeout
    timeout: u64,

    /// Args
    #[arg()]
    args: Vec<PathBuf>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Mode {
    Send,
    Receive,
}

pub fn run() {
    let args = Cli::parse();
    match args.mode {
        Mode::Send => {
            let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
            let receiver = receiver_addr(socket, &args.password, args.timeout)
                .expect("Faild to get receiver address.");
            thread::sleep(Duration::from_secs(2));
            let stream = TcpStream::connect(receiver);
            if stream.is_err() {
                eprintln!(".....Faild to Connect.....");
                return;
            } else {
                println!(".....Connect Success.....");
            }
            let mut stream = stream.unwrap();
            /*
            if args.args.is_empty() {
                stream.send_empty().unwrap();
            }*/
            //let mut wrt = BufWriter::new(stream);
            for line in std::io::stdin().lines() {
                match line {
                    Ok(input) => {
                        stream.send_msg(input).unwrap();
                        //wrt.write_all(input.as_bytes()).unwrap();
                        stream.write_all(b"\n").unwrap();
                        //wrt.flush().unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error reading input: {}", e);
                        break;
                    }
                }
            }
        }
        Mode::Receive => {
            let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();
            let addr = socket.local_addr().unwrap();
            let sender = sender_addr(socket, &args.password, args.timeout)
                .expect("Faild to get receiver address.");
            let listener = TcpListener::bind(addr).unwrap();
            let stream = get_sender_stream(listener, sender);
            if stream.is_none() {
                eprintln!(".....Faild to Connect.....");
                return;
            } else {
                println!(".....Connect Success.....");
            }
            let mut stream = stream.unwrap();
            loop {
                if !stream.receive().unwrap() {
                    break;
                }
            }
            //stream.send("This is massege".to_owned().into()).unwrap();
            /*
            let rdr = BufReader::new(stream);
            let mut stdout = std::io::stderr();
            for line in rdr.lines() {
                writeln!(stdout, "{}", line.unwrap()).unwrap();
            }*/
        }
    }
}

#[inline]
fn receiver_addr<P: AsRef<[u8]>>(
    socket: UdpSocket,
    password: P,
    timeout: u64,
) -> Option<SocketAddr> {
    get_receiver_addr(
        socket,
        password,
        "255.255.255.255:34254",
        Duration::from_secs(timeout),
    )
    .unwrap()
}

#[inline]
fn sender_addr<P: AsRef<[u8]>>(socket: UdpSocket, password: P, timeout: u64) -> Option<SocketAddr> {
    get_sender_addr(
        socket,
        password,
        Duration::from_secs(timeout), // TODO: fix bug
    )
    .unwrap()
}

#[inline]
fn get_sender_stream(listener: TcpListener, _sender_addr: SocketAddr) -> Option<TcpStream> {
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                if let Ok(_addr) = stream.local_addr() {
                    // TODO: verify authentic addr
                    return Some(stream);
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }
    None
}
