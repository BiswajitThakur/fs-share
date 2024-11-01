mod utils;

use core::panic;
use std::{
    borrow::Cow,
    fs,
    io::{self, BufReader, BufWriter, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::Path,
    time::Duration,
};

use sha2::{Digest, Sha256};
pub use utils::{get_receiver_addr, get_sender_addr};

pub const PORT: u16 = 34254;

pub const BRODCAST_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(225, 225, 225, 225)), 7879);

pub fn receiver_addr(name: &str, password: &[u8]) -> io::Result<Option<SocketAddr>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut buffer: [u8; 32] = [0; 32];
    let msg = format!("sr:{}:{}", name.len(), name);
    loop {
        socket.send_to(, addr);
    }
    todo!()
}

pub fn create_tcp_connection<F: Fn(TcpStream) -> io::Result<()>>(
    addr: Address,
    f: F,
) -> io::Result<()> {
    let listener = TcpListener::bind(addr.receiver)?;
    for stream in listener.incoming() {
        match stream {
            Ok(v) => {
                // TODO: verify authentication
                f(v)?;
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Address {
    pub sender: SocketAddr,
    pub receiver: SocketAddr,
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct SenderFs {
    user: Option<String>,
    password: Option<Vec<u8>>,
    receiver: Option<SocketAddr>,
    stream: Option<TcpStream>,
}

impl SenderFs {
    pub fn new(user: String, password: Option<Vec<u8>>) -> Self {
        Self {
            user: Some(user),
            password,
            ..Default::default()
        }
    }
    #[inline(always)]
    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.stream.as_ref().unwrap().read_exact(buf)
    }
    pub fn get_stream(&self) -> Option<&TcpStream> {
        if let Some(ref stream) = self.stream {
            Some(stream)
        } else {
            None
        }
    }
    pub fn connect(self, addr: SocketAddr) -> io::Result<Self> {
        if let Some(ref password) = self.password {
            let rev: Vec<u8> = password.iter().rev().map(|v| *v).collect();
            let mut hasher = Sha256::new();
            hasher.update(rev);
            let pass_hash = hasher.finalize().to_vec();
            let mut stream = TcpStream::connect(addr)?;
            stream.write_all(&pass_hash)?;
            let mut buf = [0; 7];
            let success_msg = b"success";
            stream.read_exact(&mut buf)?;
            if &buf == success_msg {
                Ok(Self {
                    stream: Some(stream),
                    ..self
                })
            } else {
                Ok(self)
            }
        } else {
            let stream = TcpStream::connect(addr)?;
            Ok(Self {
                stream: Some(stream),
                ..self
            })
        }
    }
    #[inline(always)]
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
    pub fn buf_writer(&self) -> Option<BufWriter<&TcpStream>> {
        if let Some(w) = &self.stream {
            Some(BufWriter::new(w))
        } else {
            None
        }
    }
    pub fn set_password(self, password: Vec<u8>) -> Self {
        Self {
            password: Some(password),
            ..self
        }
    }
    pub fn get_password(&self) -> Option<&[u8]> {
        if let Some(ref password) = self.password {
            Some(password)
        } else {
            None
        }
    }
    pub fn send(&mut self, value: SenderOps) -> io::Result<bool> {
        if self.stream.is_none() {
            return Ok(false);
        }
        let mut stream = self.stream.as_ref().unwrap();
        let mut buf = [0u8; 4];
        match value {
            // su:<user_len>:<user>:
            SenderOps::UserInfo { user } => {
                if user.is_none() {
                    return Ok(false);
                }
                let user = user.as_ref().unwrap();
                stream.write_all(format!("su:{}:", user.len()).as_bytes())?;
                stream.write_all(user.as_bytes())?;
                stream.write_all(b":")?;
            }
            // sf:<name_len>:<size>:<name>:<file>:
            SenderOps::File {
                name,
                len,
                mut reader,
            } => {
                stream.write_all(b"sf:")?;
                let f_name = name.display().to_string();
                stream.write_all(format!("{}:{}:", f_name.len(), len).as_bytes())?;
                stream.write_all(f_name.as_bytes())?;
                stream.write_all(b":")?;
                let mut buffer: [u8; 1024 * 8] = [0; 1024 * 8];
                loop {
                    let r = reader.read(&mut buffer)?;
                    if r == 0 {
                        break;
                    }
                    stream.write_all(&buffer[..r])?;
                }
                stream.write_all(b":")?;
            }
            // sm:<len>:<msg>:
            SenderOps::Msg(v) => {
                stream.write_all(b"sm:")?;
                stream.write_all(format!("{}:", v.len()).as_bytes())?;
                stream.write_all(v.as_bytes())?;
                stream.write_all(b":")?;
            }
        }
        stream.read_exact(&mut buf)?;
        if &buf != b"done" {
            panic!("Faild to send....");
        }
        Ok(true)
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct ReceiverFs {
    user: Option<String>,
    password: Option<Vec<u8>>,
    stream: Option<TcpStream>,
}

impl ReceiverFs {
    pub fn new(user: String, password: Option<Vec<u8>>) -> Self {
        Self {
            user: Some(user),
            password,
            ..Default::default()
        }
    }
    pub fn set_password(self, password: Vec<u8>) -> Self {
        Self {
            password: Some(password),
            ..self
        }
    }
    pub fn get_password(&self) -> Option<&[u8]> {
        if let Some(ref password) = self.password {
            Some(password)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn get_stream(self) -> Option<TcpStream> {
        self.stream
    }
    pub fn connect_sender(self, listener: TcpListener, limit: usize) -> io::Result<Self> {
        let mut count: usize = 0;
        for stream in listener.incoming() {
            count += 1;
            let mut stream = stream?;
            if self.verify_passw(&mut stream) {
                stream.write_all(b"success")?;
                return Ok(Self {
                    stream: Some(stream),
                    ..self
                });
            } else {
                stream.write_all(b"wrongpw")?;
                eprintln!(
                    "Skipped : '{}': Invalid Password send",
                    stream.local_addr()?
                );
                if count >= limit {
                    return Ok(self);
                }
                continue;
            }
        }
        Ok(self)
    }
    #[inline(always)]
    pub fn is_sender_connected(&self) -> bool {
        self.stream.is_some()
    }
    #[inline(always)]
    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.stream.as_ref().unwrap().read_exact(buf)
    }
    pub fn receive(&mut self) -> io::Result<ReceiverOps> {
        if !self.is_sender_connected() {
            return Ok(ReceiverOps::None);
        }
        let mut stream = self.stream.as_ref().unwrap();
        let mut header_buf: [u8; 3] = [0; 3];
        let mut buffer: [u8; 1024 * 16] = [0; 1024 * 16];
        let mut buf: [u8; 1] = [0; 1];
        stream.read_exact(&mut header_buf)?;
        match &header_buf {
            b"sm:" => {
                let mut msg_len = 0usize;
                let mut readed = 0usize;
                loop {
                    stream.read_exact(&mut buf)?;
                    if buf[0] == b':' {
                        break;
                    }
                    msg_len *= 10;
                    msg_len += buf[0] as usize - b'0' as usize;
                }
                let mut s = String::with_capacity(msg_len);
                loop {
                    let r = stream.read(&mut buffer)?;
                    readed += r;
                    if readed <= msg_len {
                        s.push_str(&String::from_utf8_lossy(&buffer[..r]));
                    } else {
                        s.push_str(&String::from_utf8_lossy(&buffer[..r - 1]));
                        if buffer[r - 1] != b':' {
                            panic!("Msg end with unexpected char...");
                        }
                        break;
                    }
                }
                stream.write_all(b"done")?;
                return Ok(ReceiverOps::Msg(s.into()));
            }
            b"sf:" => {
                // sf:<name_len>:<size>:<name>:<file>:
                let mut name_len = 0usize;
                let mut file_len = 0usize;
                let mut readed = 0usize;
                loop {
                    stream.read_exact(&mut buf)?;
                    if buf[0] == b':' {
                        break;
                    }
                    name_len *= 10;
                    name_len += buf[0] as usize - b'0' as usize;
                }
                loop {
                    stream.read_exact(&mut buf)?;
                    if buf[0] == b':' {
                        break;
                    }
                    file_len *= 10;
                    file_len += buf[0] as usize - b'0' as usize;
                }
                let mut i = 0;
                let mut f_name = String::with_capacity(name_len);
                while i < name_len {
                    stream.read_exact(&mut buf)?;
                    f_name.push(buf[0] as char);
                    i += 1;
                }
                stream.read_exact(&mut buf)?;
                if buf[0] != b':' {
                    panic!("Unexpected end of file name...");
                }
                let f = fs::File::create("/sdcard/file.mkv")?;
                let mut buf_writer = BufWriter::new(f);
                loop {
                    let r = stream.read(&mut buffer)?;
                    readed += r;
                    if readed <= file_len {
                        buf_writer.write_all(&buffer[..r])?;
                        print!("File receiving: {}%\r", (readed / file_len) * 100);
                    } else {
                        buf_writer.write_all(&buffer[..r - 1])?;
                        if buffer[r - 1] != b':' {
                            panic!("Unexpected end of file...");
                        }
                        print!("File receiving: {}%\r", ((readed - 1) / file_len) * 100);
                        std::io::stdout().flush()?;
                        break;
                    }
                    std::io::stdout().flush()?;
                }
                stream.write_all(b"done")?;
                return Ok(ReceiverOps::File {
                    name: Path::new("tmp.file").into(),
                    size: file_len,
                });
            }
            _ => Ok(ReceiverOps::None),
        }
    }
    fn verify_passw(&self, stream: &mut TcpStream) -> bool {
        let mut buffer = [0; 32];
        if let Some(ref pass) = self.password {
            let rev: Vec<u8> = pass.iter().rev().map(|v| *v).collect();
            let mut hasher = Sha256::new();
            hasher.update(rev);
            let want = hasher.finalize().to_vec();
            match stream.read_exact(&mut buffer) {
                Ok(_) => {
                    if buffer == *want {
                        return true;
                    } else {
                        return false;
                    }
                }
                Err(_) => false,
            }
        } else {
            match stream.read_exact(&mut buffer) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }
}

pub enum SenderOps<'a> {
    UserInfo {
        user: Option<Cow<'a, str>>,
    },
    File {
        name: Cow<'a, Path>,
        len: usize,
        reader: Box<BufReader<dyn io::Read>>,
    },
    Msg(Cow<'a, str>),
}
#[derive(Debug, PartialEq, Eq)]
pub enum ReceiverOps<'a> {
    None,
    UserInfo { user: Option<Cow<'a, str>> },
    File { name: Cow<'a, Path>, size: usize },
    Msg(Cow<'a, str>),
}
