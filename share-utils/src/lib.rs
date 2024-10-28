mod utils;

use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use sha2::{Digest, Sha256};
pub use utils::{get_receiver_addr, get_sender_addr};

#[allow(unused)]
#[derive(Debug)]
pub struct FileInfo {
    name: String,
    size: usize,
}

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
            let success_msg = [b's', b'u', b'c', b'c', b'e', b's', b's'];
            stream.read_exact(&mut buf)?;
            if buf == success_msg {
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

/*
#[cfg(test)]
mod tests {
    use crate::{ReceiverFs, SenderFs};
    use std::{
        io::{Read, Write},
        net::{IpAddr, Ipv4Addr, SocketAddr},
    };

    #[test]
    fn test_connect() {
        let receiver_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let receiver = ReceiverFs::try_from(receiver_addr);
        assert!(receiver.is_ok());
        let receiver = receiver.unwrap();
        let port = receiver.receiver_port();
        assert!(port.is_some());
        let port = port.unwrap();
        assert!(port != 0);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        assert_eq!(receiver.receiver_addr().unwrap(), addr);
        let mut sender = SenderFs::default();
        assert!(sender.connect(receiver_addr).is_err());
        assert!(sender.connect(addr).is_ok());
        let w: [u8; 5] = [3; 5];
        let mut r: [u8; 5] = [0; 5];
        {
            let mut bf_writer = sender.buf_writer().unwrap();
            bf_writer.write_all(&w).unwrap();
        }
        {
            let mut bf_reader = receiver.buf_reader().unwrap().unwrap();
            bf_reader.read_exact(&mut r).unwrap();
        }
        assert_eq!(r, w);
    }
}
*/
