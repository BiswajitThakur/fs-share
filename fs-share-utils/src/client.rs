use std::{
    io,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket},
    os::unix::process,
    thread::{self, Thread},
    time::{self, Duration},
};

pub struct Client<T: io::Read + io::Write> {
    stream: T,
}

impl<T: io::Read + io::Write> io::Write for Client<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.stream.write_all(buf)
    }
}

impl<T: io::Read + io::Write> io::Read for Client<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.stream.read_exact(buf)
    }
}
