use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use clap::{Parser, ValueEnum};
use share_utils::{get_receiver_addr, get_sender_addr};

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
            let stream = Arc::new(Mutex::new(stream.unwrap()));
            let stream_for_write = Arc::clone(&stream);
            let stream_for_read = Arc::clone(&stream);
            thread::spawn(move || {
                let stream_locked = stream_for_write.lock().unwrap();
                let mut wrt = BufWriter::new(&*stream_locked);
                for line in std::io::stdin().lines() {
                    let line = line.unwrap();
                    wrt.write_all(line.as_bytes()).unwrap();
                    wrt.write_all(b"\n").unwrap();
                    wrt.flush().unwrap();
                }
            });
            thread::spawn(move || {
                let stream_locked = stream_for_read.lock().unwrap();
                let rdr = BufReader::new(&*stream_locked);
                let mut stdout = std::io::stdout();
                for line in rdr.lines() {
                    let line = line.unwrap();
                    writeln!(stdout, "{}", line).unwrap();
                }
            });
            loop {
                thread::park();
            }
            /*
            for line in std::io::stdin().lines() {
                stream.write_all(line.unwrap().as_bytes()).unwrap();
                stream.write_all(b"\n").unwrap();
            }*/
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
            let stream = Arc::new(Mutex::new(stream.unwrap()));
            let stream_for_write = Arc::clone(&stream);
            let stream_for_read = Arc::clone(&stream);
            thread::spawn(move || {
                let stream_locked = stream_for_write.lock().unwrap();
                let mut wrt = BufWriter::new(&*stream_locked);
                for line in std::io::stdin().lines() {
                    let line = line.unwrap();
                    wrt.write_all(line.as_bytes()).unwrap();
                    wrt.write_all(b"\n").unwrap();
                    wrt.flush().unwrap();
                }
            });
            thread::spawn(move || {
                let stream_locked = stream_for_read.lock().unwrap();
                let rdr = BufReader::new(&*stream_locked);
                let mut stdout = std::io::stdout();
                for line in rdr.lines() {
                    let line = line.unwrap();
                    writeln!(stdout, "{}", line).unwrap();
                }
            });
            loop {
                thread::park();
            }
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
