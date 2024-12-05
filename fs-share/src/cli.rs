use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    thread,
    time::Duration,
};

use clap::{Parser, ValueEnum};
use share_utils::{get_receiver_addr, get_sender_addr, Colorize, ShareFs};

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

    /// port
    #[arg(long, default_value_t = 34254)]
    port: u16,

    /// Timeout
    #[arg(long, default_value_t = 60)]
    timeout: u64,

    // /// Outpit dir
    // #[arg(long)]
    // out: PathBuf,
    /// Args
    #[arg()]
    args: Vec<PathBuf>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Mode {
    Send,
    Receive,
}

pub fn run() -> io::Result<()> {
    let args = Cli::parse();
    let mut stdout = std::io::stdout();
    match args.mode {
        Mode::Send => {
            let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
            let receiver = receiver_addr(socket, &args.password, args.timeout, args.port)
                .unwrap_or_else(|| panic!("{}", "Faild to get receiver address.".bold().red()));
            thread::sleep(Duration::from_secs(2));
            let stream = TcpStream::connect(receiver);
            if stream.is_err() {
                eprintln!("{}", ".....Faild to Connect.....".bold().green());
                return Ok(());
            } else {
                println!("{}", ".....Connect Success.....".bold().green());
            }
            let mut stream = stream.unwrap();

            for file in args.args {
                if file.is_file() {
                    stream.send_file(file, &mut stdout)?;
                }
            }
            stream.send_eof()?;
            loop {
                if !stream.receive(&mut stdout).unwrap() {
                    break;
                }
            }
        }
        Mode::Receive => {
            let socket = UdpSocket::bind(format!("0.0.0.0:{}", args.port)).unwrap();
            let addr = socket.local_addr().unwrap();
            let sender = sender_addr(socket, &args.password, args.timeout)
                .unwrap_or_else(|| panic!("{}", "Faild to get receiver address.".bold().red()));
            let listener = TcpListener::bind(addr).unwrap();
            let stream = get_sender_stream(listener, sender);
            if stream.is_none() {
                eprintln!("{}", ".....Faild to Connect.....".bold().red());
                return Ok(());
            } else {
                println!("{}", ".....Connect Success.....".bold().green());
            }
            let mut stream = stream.unwrap();
            loop {
                if !stream.receive(&mut stdout).unwrap() {
                    break;
                }
            }
            for file in args.args {
                stream.send_file(file, &mut stdout)?;
            }
            stream.send_eof()?;
        }
    }
    Ok(())
}

#[inline]
fn receiver_addr<P: AsRef<[u8]>>(
    socket: UdpSocket,
    password: P,
    timeout: u64,
    port: u16,
) -> Option<SocketAddr> {
    get_receiver_addr(
        socket,
        password,
        format!("255.255.255.255:{}", port),
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
