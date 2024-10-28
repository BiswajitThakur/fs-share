mod utils;

use std::{
    borrow::Cow,
    fs,
    io::{self, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
pub use utils::{get_receiver_addr, get_sender_addr};

pub const PORT: u16 = 34254;

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
    pub fn get_stream(&self) -> Option<&TcpStream> {
        if let Some(ref stream) = self.stream {
            Some(stream)
        } else {
            None
        }
    }
    pub fn connect(&mut self, addr: SocketAddr) -> io::Result<bool> {
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
                self.stream = Some(stream);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            let stream = TcpStream::connect(addr)?;
            self.stream = Some(stream);
            Ok(true)
        }
    }
    pub fn buf_writer(&self) -> Option<BufWriter<&TcpStream>> {
        if let Some(w) = &self.stream {
            Some(BufWriter::new(w))
        } else {
            None
        }
    }
    pub fn set_password(&mut self, password: Vec<u8>) {
        self.password = Some(password);
    }
    pub fn get_password(&self) -> Option<&[u8]> {
        if let Some(ref password) = self.password {
            Some(password)
        } else {
            None
        }
    }
    pub fn send(&mut self, value: &SenderOps) -> io::Result<bool> {
        if self.stream.is_none() {
            return Ok(false);
        }
        let mut stream = self.stream.as_ref().unwrap();
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
            SenderOps::File { name } => {
                let f = fs::File::open(name)?;
                let f_size = f.metadata()?.len();
                stream.write_all(b"sf:")?;
                let f_name = name.display().to_string();
                stream.write_all(format!("{}:{}:", f_name.len(), f_size).as_bytes())?;
                stream.write_all(f_name.as_bytes())?;
                stream.write_all(b":")?;
                let mut rdr = BufReader::new(f);
                let mut buffer: [u8; 4096] = [0; 4096];
                loop {
                    let r = rdr.read(&mut buffer)?;
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
        Ok(true)
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct ReceiverFs {
    user: Option<String>,
    password: Option<Vec<u8>>,
    sender: Option<SocketAddr>,
    listener: Option<TcpListener>,
}

impl ReceiverFs {
    pub fn new(user: String, password: Option<Vec<u8>>) -> Self {
        Self {
            user: Some(user),
            password,
            ..Default::default()
        }
    }
    pub fn set_password(&mut self, password: Vec<u8>) {
        self.password = Some(password);
    }
    pub fn get_password(&self) -> Option<&[u8]> {
        if let Some(ref password) = self.password {
            Some(password)
        } else {
            None
        }
    }
    pub fn bind(&mut self, addr: SocketAddr) -> io::Result<()> {
        self.listener = Some(TcpListener::bind(addr)?);
        Ok(())
    }
    pub fn connect_sender(&self, limit: usize) -> io::Result<Option<TcpStream>> {
        if let Some(ref listener) = self.listener {
            let mut count = 0;
            for stream in listener.incoming() {
                count += 1;
                let mut stream = stream?;
                if self.verify_passw(&mut stream) {
                    stream.write_all(b"success")?;
                    return Ok(Some(stream));
                } else {
                    stream.write_all(b"wrongpw")?;
                    eprintln!(
                        "Skipped : '{}': Invalid Password send",
                        stream.local_addr()?
                    );
                    if count >= limit {
                        return Ok(None);
                    }
                    continue;
                }
            }
            Ok(None)
        } else {
            Ok(None)
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

    pub fn receiver_port(&self) -> Option<u16> {
        if let Some(ref listener) = self.listener {
            Some(listener.local_addr().ok()?.port())
        } else {
            None
        }
    }
    pub fn receiver_addr(&self) -> Option<SocketAddr> {
        if let Some(ref listener) = self.listener {
            Some(listener.local_addr().ok()?)
        } else {
            None
        }
    }
    /*
    pub fn buf_reader(&self) -> io::Result<Option<BufReader<TcpStream>>> {
        if let Some(listener) = &self.listener {
            //for stream in listener.incoming() {
            if let Some(stream) = listener.incoming().next() {
                return Ok(Some(BufReader::new(stream?)));
            }
            Ok(None)
        } else {
            Ok(None)
        }
    }
    */
}

impl TryFrom<SocketAddr> for ReceiverFs {
    type Error = io::Error;
    fn try_from(value: SocketAddr) -> Result<Self, Self::Error> {
        Ok(Self {
            listener: Some(TcpListener::bind(value)?),
            ..Default::default()
        })
    }
}

#[allow(unused)]
pub enum SenderOps<'a> {
    UserInfo { user: Option<Cow<'a, str>> },
    File { name: Cow<'a, Path> },
    Msg(Cow<'a, str>),
}

/*
impl TryFrom<&TcpStream> for SenderOps {
    type Error = io::Error;
    fn try_from(mut value: &TcpStream) -> Result<Self, Self::Error> {
        let mut buf_info: [u8; 1] = [0; 1];
        value.read_exact(&mut buf_info)?;
        match buf_info[0] {
            b'1' => {

            }
            _ => todo!(),
        };
        todo!()
    }
}
*/
