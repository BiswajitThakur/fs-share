mod addr;
mod client;
mod utils;

use argon2::Argon2;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub use addr::get_sender_addr;
use rand::Rng;
use serde::ser::SerializeStruct;
use std::ffi::OsString;
use std::fmt::{self, Write as FWrite};
use std::io::{BufRead, BufWriter, Cursor, Read};
use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener, ToSocketAddrs, UdpSocket};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::Thread;
use std::time::{self, Duration};
use std::{cmp, fs, thread};
use std::{
    fs::File,
    io::{self, BufReader, Write},
    net::TcpStream,
    path::Path,
};
use utils::create_file_path;

pub use colored::*;

pub use utils::sha256;

const SEND_BUFFER_SIZE: usize = 32 * 1024;
const RECEIVE_BUFFER_SIZE: usize = 32 * 1024;

impl ShareFs for TcpStream {}

pub trait ShareFs: Sized + io::Write + io::Read {
    fn send_eof(&mut self) -> io::Result<()> {
        write!(self, ":00:")
    }
    fn send_msg<T: AsRef<str>>(&mut self, value: T) -> io::Result<()> {
        let msg = value.as_ref();
        write!(self, "msg:{}:", msg.as_bytes().len())?;
        let mut cursor = Cursor::new(msg);
        transfer_without_progress(&mut cursor, self, msg.len(), SEND_BUFFER_SIZE)?;
        let mut v = Vec::with_capacity(4);
        transfer_without_progress(self, &mut v, 4, 4)?;
        let prefix = std::str::from_utf8(&v).unwrap_or_default();
        match prefix {
            ":ss:" => {}
            _ => {
                eprintln!(".....Falid to Send.....");
                std::process::exit(1);
            }
        }
        Ok(())
    }
    fn receive_msg<W: io::Write>(&mut self, stdout: &mut W) -> io::Result<()> {
        let len = read_num(self)?;
        transfer_without_progress(self, stdout, len, RECEIVE_BUFFER_SIZE)?;
        stdout.write_all(b"\n")?;
        self.write_all(b":ss:")?;
        Ok(())
    }
    fn receive_file_with_progress<W: io::Write>(&mut self, stdout: &mut W) -> io::Result<()> {
        let name_length = read_num(self)?;
        let file_length = read_num(self)?;
        let mut name: Vec<u8> = Vec::with_capacity(name_length);
        transfer_without_progress(self, &mut name, name_length, cmp::min(name_length, 255))?;
        let name = std::str::from_utf8(&name).unwrap_or("unknown");
        let file_path = create_file_path(name);
        let file = fs::File::create_new(file_path)?;
        let mut file_writer = BufWriter::new(file);
        writeln!(
            stdout,
            "File Receiving: {}, Size: {} bytes",
            name.yellow().bold(),
            file_length.to_string().green().bold()
        )?;
        transfer_with_progress(self, &mut file_writer, file_length, RECEIVE_BUFFER_SIZE)?;
        file_writer.flush()?;
        self.write_all(b":ss:")?;
        Ok(())
    }
    fn get_prefix(&mut self) -> io::Result<String> {
        let mut v: Vec<u8> = Vec::with_capacity(4);
        transfer_without_progress(self, &mut v, 4, 4)?;
        let prefix = String::from_utf8(v).unwrap_or_default();
        Ok(prefix)
    }
    fn receive<W: io::Write>(&mut self, stdout: &mut W) -> io::Result<bool> {
        match self.get_prefix()?.as_str() {
            ":00:" => Ok(false),
            "msg:" => {
                self.receive_msg(stdout)?;
                Ok(true)
            }
            "fff:" => {
                self.receive_file_with_progress(stdout)?;
                Ok(true)
            }
            p => {
                eprintln!(
                    "{}: unknown prefix: {}",
                    "ERROR".red().bold(),
                    p.red().bold()
                );
                std::process::exit(1);
            }
        }
    }
    fn send_file<P: AsRef<Path>, W: io::Write>(
        &mut self,
        file: P,
        stdout: &mut W,
    ) -> io::Result<()> {
        let f_name = file
            .as_ref()
            .display()
            .to_string()
            .replace("\"", "")
            .replace("'", "");
        let f = File::open(file)?;
        let file_len = f.metadata()?.len();
        writeln!(
            stdout,
            "Sending file: {}, size: {} bytes",
            &f_name, file_len
        )?;
        write!(
            self,
            "fff:{}:{}:{}",
            f_name.as_bytes().len(),
            file_len,
            f_name
        )?;
        let mut reader = BufReader::new(f);
        transfer_with_progress(&mut reader, self, file_len as usize, SEND_BUFFER_SIZE)?;
        let mut buffer: Vec<u8> = vec![0; 4];
        self.read_exact(&mut buffer[0..4])?;
        let prefix = std::str::from_utf8(&buffer[0..4]).unwrap_or_default();
        match prefix {
            ":ss:" => {}
            _ => {
                writeln!(stdout, "{}", ".....Falid to Send.....".bold().red())?;
                std::process::exit(1);
            }
        }
        Ok(())
    }
}

pub fn transfer_without_progress<R: io::Read, W: io::Write>(
    r: &mut R,
    w: &mut W,
    n: usize,
    buf_size: usize,
) -> io::Result<()> {
    let mut remain = n;
    let mut buffer = vec![0; std::cmp::min(n, buf_size)];

    while remain > 0 {
        let to_read = std::cmp::min(buffer.len(), remain);
        let read_count = r.read(&mut buffer[..to_read])?;
        if read_count == 0 {
            break;
        }
        w.write_all(&buffer[..read_count])?;
        remain -= read_count;
    }
    Ok(())
}

fn transfer_with_progress<R: io::Read, W: io::Write>(
    r: &mut R,
    w: &mut W,
    n: usize,
    buf_size: usize,
) -> io::Result<()> {
    let mut remain = n;
    let mut buffer = vec![0; std::cmp::min(n, buf_size)];

    let mut i = 0;
    let pb = ProgressBar::new(n as u64);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn FWrite| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    while remain > 0 {
        let to_read = std::cmp::min(buffer.len(), remain);
        let read_count = r.read(&mut buffer[..to_read])?;
        if read_count == 0 {
            break;
        }
        w.write_all(&buffer[..read_count])?;
        remain -= read_count;

        i += read_count as u64;
        pb.set_position(i);
    }
    pb.finish();
    Ok(())
}

pub fn read_num<R: io::Read>(r: &mut R) -> io::Result<usize> {
    let mut buffer = [0; 1];
    let mut num = 0;
    loop {
        let c = r.read(&mut buffer)?;
        if c == 0 || !buffer[0].is_ascii_digit() {
            break;
        }
        if num > usize::MAX / 10 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Number too large",
            ));
        }
        num = num * 10 + (buffer[0] - b'0') as usize;
    }
    Ok(num)
}

pub enum Status {
    Success,
    VersionNotMatch,
}

impl TryFrom<&[u8; 1]> for Status {
    type Error = ();
    fn try_from(value: &[u8; 1]) -> Result<Self, Self::Error> {
        match &value {
            [0] => Ok(Self::Success),
            [1] => Ok(Self::VersionNotMatch),
            _ => Err(()),
        }
    }
}

impl Status {
    fn as_bytes(&self) -> &[u8; 1] {
        match self {
            Self::Success => &[0],
            Self::VersionNotMatch => &[1],
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
    pub port: u16,
    pub ipv6: Ipv6Addr, // 128 bit
}

impl Default for Header {
    fn default() -> Self {
        Self {
            version: 0,
            port: 0,
            ipv6: Ipv6Addr::LOCALHOST,
        }
    }
}

impl From<&Header> for [u8; 20] {
    // 16 + 16 + 128 = 160
    // 160 / 8 = 20
    fn from(value: &Header) -> Self {
        let version = value.version.to_be_bytes();
        let port = value.port.to_be_bytes();
        let mut buffer = [0; 20];
        buffer[0] = version[0];
        buffer[1] = version[1];
        buffer[2] = port[0];
        buffer[3] = port[1];
        let mut iter = value.ipv6.to_bits().to_be_bytes().into_iter();
        for u in (&mut buffer[4..]).iter_mut() {
            *u = iter.next().unwrap();
        }
        buffer
    }
}

impl From<[u8; 20]> for Header {
    fn from(v: [u8; 20]) -> Self {
        let version = u16::from_be_bytes([v[0], v[1]]);
        let port = u16::from_be_bytes([v[2], v[3]]);
        let mut ip = [0; 16];
        for (index, u) in (&v[4..]).iter().enumerate() {
            ip[index] = *u;
        }
        let ipv6 = Ipv6Addr::from_bits(u128::from_be_bytes(ip));
        Self {
            version,
            port,
            ipv6,
        }
    }
}

impl Header {
    #[inline(always)]
    fn to_be_bytes(&self) -> [u8; 20] {
        self.into()
    }
    fn to_addr(&self) -> SocketAddr {
        SocketAddr::V6(SocketAddrV6::new(self.ipv6, self.port, 0, 1))
    }
}

#[cfg(test)]
mod tests1 {
    use std::io::{self, sink, Cursor, Read};

    use crate::{read_num, transfer_with_progress, transfer_without_progress};
    const BUFFER_SIZE: usize = 1024;

    #[test]
    fn test_read_num_valid() {
        let data = b"123045:abc";
        let mut cursor = Cursor::new(data);
        let result = read_num(&mut cursor).unwrap();
        assert_eq!(result, 123045);
    }
    #[test]
    fn test_read_num_empty() {
        let data = b"";
        let mut cursor = Cursor::new(data);
        let result = read_num(&mut cursor).unwrap();
        assert_eq!(result, 0);
    }
    #[test]
    fn test_read_num_with_leading_zeros() {
        let data = b"000000000000000000000000000000123";
        let mut cursor = Cursor::new(data);
        let result = read_num(&mut cursor).unwrap();
        assert_eq!(result, 123);
    }
    #[test]
    fn test_read_num_very_large_number() {
        let data = format!("00000000{}::::::", usize::MAX);
        let mut cursor = Cursor::new(data);
        let result = read_num(&mut cursor).unwrap();
        assert_eq!(result, usize::MAX);
    }
    #[test]
    fn test_read_num_too_large_number() {
        let data = b"9999999999999999999999999999999999";
        let mut cursor = Cursor::new(data);
        let result = read_num(&mut cursor);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }
    #[test]
    fn test_read_num() {
        let data = b"0000100xyz43210?1*";
        let mut cursor = Cursor::new(data);
        let result = read_num(&mut cursor).unwrap(); // x
        assert_eq!(result, 100);
        let result = read_num(&mut cursor).unwrap(); // y
        assert_eq!(result, 0);
        let result = read_num(&mut cursor).unwrap(); // z
        assert_eq!(result, 0);
        let result = read_num(&mut cursor).unwrap(); // ?
        assert_eq!(result, 43210);
        let result = read_num(&mut cursor).unwrap(); // *
        assert_eq!(result, 1);
    }

    #[test]
    fn test_read_n_bytes_exact() {
        let data = b"1234567890";
        let mut reader = Cursor::new(data);
        let mut writer = Vec::new();

        // Read exactly 10 bytes
        transfer_without_progress(&mut reader, &mut writer, 10, BUFFER_SIZE).unwrap();
        assert_eq!(writer, data);
    }
    #[test]
    fn test_read_n_bytes_partial() {
        let data = b"1234567890";
        let mut reader = Cursor::new(data);
        let mut writer = Vec::new();

        // Read only 5 bytes out of the 10 available
        transfer_without_progress(&mut reader, &mut writer, 5, BUFFER_SIZE).unwrap();
        assert_eq!(writer, b"12345");
    }
    #[test]
    fn test_read_n_bytes_large_n() {
        let data = b"1234567890";
        let mut reader = Cursor::new(data);
        let mut writer = Vec::new();

        // Attempt to read more bytes than available
        transfer_without_progress(&mut reader, &mut writer, 20, BUFFER_SIZE).unwrap();
        assert_eq!(writer, data); // Writer should contain all available bytes
    }
    #[test]
    fn test_read_n_bytes_empty_input() {
        let data = b"";
        let mut reader = Cursor::new(data);
        let mut writer = Vec::new();

        // Reading from an empty source should not fail
        transfer_without_progress(&mut reader, &mut writer, 10, BUFFER_SIZE).unwrap();
        assert!(writer.is_empty()); // Writer should remain empty
    }
    #[test]
    fn test_read_n_bytes_zero_bytes() {
        let data = b"1234567890";
        let mut reader = Cursor::new(data);
        let mut writer = Vec::new();

        // Reading 0 bytes should result in no data written
        transfer_without_progress(&mut reader, &mut writer, 0, BUFFER_SIZE).unwrap();
        assert!(writer.is_empty());
    }
    #[test]
    fn test_read_n_bytes_sink() {
        let data = b"1234567890";
        let mut reader = Cursor::new(data);
        let mut writer = sink(); // /dev/null equivalent

        // Reading into a sink; ensures no errors occur
        transfer_without_progress(&mut reader, &mut writer, 5, BUFFER_SIZE).unwrap();
        // Nothing to verify in the writer since it's a sink
    }
    #[test]
    #[ignore = "reason"]
    fn test_read_n_bytes_sink_large_data() {
        use std::io::repeat;

        // Simulate a source of infinite data (a stream of the byte `b'x'`)
        let data = repeat(b'x'); // Infinite data
        let mut reader = data.take(5 * 1024 * 1024 * 1024); // Limit to 5 GB
        let mut writer = sink(); // Write to a sink (discarding data)

        // Read 1.5 GB of data and write it to the sink
        transfer_with_progress(
            &mut reader,
            &mut writer,
            3 * 1024 * 1024 * 1024 / 2,
            BUFFER_SIZE,
        )
        .unwrap();

        // Test completes if no error occurs during the read/write
    }
    #[test]
    fn test_read_n_bytes() {
        let data = String::from_iter((0..20).into_iter().map(|_| {
            format!(
                "{}{}{}",
                String::from_iter('a'..='z'),
                String::from_iter('A'..='Z'),
                String::from_iter('0'..='9')
            )
        }));
        let mut reader = Cursor::new(data); // length = ((2 * 26) + 10) * 20 == 1240
        let mut writer = Vec::new();
        transfer_without_progress(&mut reader, &mut writer, 1, BUFFER_SIZE).unwrap();
        assert_eq!(writer, b"a");
        let mut writer = Vec::new();
        transfer_without_progress(&mut reader, &mut writer, 15, BUFFER_SIZE).unwrap();
        assert_eq!(writer, b"bcdefghijklmnop");
        transfer_without_progress(&mut reader, &mut writer, 5, BUFFER_SIZE).unwrap();
        let mut writer = Vec::new();
        transfer_without_progress(&mut reader, &mut writer, 15, BUFFER_SIZE).unwrap();
        assert_eq!(writer, b"vwxyzABCDEFGHIJ");
        transfer_without_progress(&mut reader, &mut writer, (1240 - 36) - 20, BUFFER_SIZE).unwrap();
        let mut writer = Vec::new();
        transfer_with_progress(&mut reader, &mut writer, 20, BUFFER_SIZE).unwrap();
        assert_eq!(writer, b"QRSTUVWXYZ0123456789");
    }
}

const APT_VERSION: u16 = 1;
const PREFIX: &'static str = ":fs-share:";

#[derive(Debug, Clone, PartialEq, Eq)]
struct UdpDataStream {
    api_version: u16,
    port: u16,
    id: u64,
    os: String,
    user_name: String,
}

/*
impl serde::Serialize for UdpDataStream {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut f = serializer.serialize_struct("UdpDataStream", 4)?;
        f.serialize_field("port", &self.port)?;
        f.serialize_field("id", &self.id)?;
        f.serialize_field("os", &self.os)?;
        f.serialize_field("user_name", &self.user_name)?;
        todo!()
    }
}
*/
impl fmt::Display for UdpDataStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "OS: {}", self.os.yellow())?;
        writeln!(f, "User: {}", self.user_name.yellow())?;
        write!(f, "ID: {}", self.id.to_string().yellow())?;
        Ok(())
    }
}

impl From<u16> for UdpDataStream {
    fn from(port: u16) -> Self {
        let os = std::env::consts::OS.to_string();
        let user_name = whoami::realname();
        let id = rand::rng().random(); // TODO: generate random u128
        Self {
            api_version: APT_VERSION,
            port,
            id,
            os,
            user_name,
        }
    }
}

impl UdpDataStream {
    fn new() {}
    fn to_be_bytes_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Prefix (Fixed 10 bytes)
        bytes.extend_from_slice(PREFIX.as_bytes());

        // API Version (2 bytes, Big Endian)
        bytes.extend_from_slice(&self.api_version.to_be_bytes());

        // TCP Port (2 bytes, Big Endian)
        bytes.extend_from_slice(&self.port.to_be_bytes());

        // ID (8 bytes, Big Endian)
        bytes.extend_from_slice(&self.id.to_be_bytes());

        // OS (16 bits length as Big Endian + data)
        bytes.extend_from_slice(&(self.os.len() as u16).to_be_bytes());
        bytes.extend_from_slice(self.os.as_bytes());

        // User Name (16 bits length as Big Endian + data)
        bytes.extend_from_slice(&(self.user_name.len() as u16).to_be_bytes());
        bytes.extend_from_slice(self.user_name.as_bytes());

        bytes.shrink_to_fit();

        bytes
    }
    fn from_be_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 22 || &bytes[..10] != b":fs-share:" {
            return None; // Invalid length or missing prefix
        }

        let api_version = u16::from_be_bytes([bytes[10], bytes[11]]);
        let port = u16::from_be_bytes([bytes[12], bytes[13]]);
        let id = u64::from_be_bytes([
            bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21],
        ]);

        let mut index = 22;

        // Extract OS string
        let os_len = u16::from_be_bytes([bytes[index], bytes[index + 1]]) as usize;
        index += 2;
        if index + os_len > bytes.len() {
            return None; // Prevent out-of-bounds access
        }
        let os = String::from_utf8_lossy(&bytes[index..index + os_len]).to_string();
        index += os_len;

        // Extract Username string
        let user_len = u16::from_be_bytes([bytes[index], bytes[index + 1]]) as usize;
        index += 2;
        if index + user_len > bytes.len() {
            return None; // Prevent out-of-bounds access
        }
        let user_name = String::from_utf8_lossy(&bytes[index..index + user_len]).to_string();

        Some(Self {
            api_version,
            port,
            id,
            os,
            user_name,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecvType {
    File(String),
    Eof,
}

pub trait RecvDataWithoutPd: BufRead {
    fn recv_data_without_pd(&mut self) -> io::Result<RecvType> {
        todo!()
    }
}

impl RecvDataWithoutPd for BufReader<TcpStream> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendData<T> {
    /// fff:\xXX\xXX<file name><64 bit size of file><...file...>\xEE\xFF
    ///     ^------^ => 16 bit length of file name
    File(T),
    Msg(String),
    Eof,
}

impl<T: AsRef<Path>> SendData<T> {
    pub fn send_without_progress<W: io::Write>(self, stream: &mut BufWriter<W>) -> io::Result<()> {
        match self {
            Self::File(path) => {
                let file_name = path
                    .as_ref()
                    .file_name()
                    .expect("not a file")
                    .to_str()
                    .unwrap();
                assert!(file_name.len() < u16::MAX as usize);
                let file = fs::File::open(&path)?;
                let file_len = file.metadata()?.len();
                let mut reader = BufReader::new(file);
                stream.write_all(b"fff:")?; // sending file
                stream.write_all(&(file_name.len() as u16).to_be_bytes())?; // sending 16 bit length of file
                stream.write_all(file_name.as_bytes())?; // sending file name
                stream.write_all(&file_len.to_be_bytes())?; // sending 64 bit length of file
                io::copy(&mut reader, stream)?; // sending the file
                stream.write_all(b"\xEE\xFF")?; // End of file
            }
            Self::Eof => stream.write_all(b":00:")?,
            _ => {}
        }
        Ok(())
    }
}

pub struct SenderFs<R: io::Write> {
    stream: BufWriter<R>,
}

pub struct ReceiverFs<W: io::Read> {
    stream: BufReader<W>,
}

impl<R: io::Read> ReceiverFs<R> {
    pub fn recv(&mut self) -> io::Result<()> {
        todo!()
    }
}

pub enum ClientType {
    Sender(SenderAddr),
    Receiver(ReceiverAddr),
}

pub struct SenderAddr {
    broadcast_addr: SocketAddr,
}

impl SenderAddr {
    pub fn set_broadcast_addr(mut self, addr: SocketAddr) -> Self {
        self.broadcast_addr = addr;
        self
    }
    pub fn build(self) -> ClientType {
        ClientType::Sender(self)
    }
}

#[derive(Debug)]
pub struct ReceiverAddr {
    tcp_listener_addr: SocketAddr,
    udp_socket_addr: SocketAddr,
    broadcast_addr: SocketAddr,
}

impl ReceiverAddr {
    pub fn set_tcp_listener_addr(mut self, addr: SocketAddr) -> Self {
        self.tcp_listener_addr = addr;
        self
    }
    pub fn set_udp_socket_addr(mut self, addr: SocketAddr) -> Self {
        self.udp_socket_addr = addr;
        self
    }
    pub fn set_broadcast_addr(mut self, addr: SocketAddr) -> Self {
        self.broadcast_addr = addr;
        self
    }
    pub fn build(self) -> ClientType {
        ClientType::Receiver(self)
    }
}

pub enum TransmissionMode<T> {
    HalfDuplex(T),
    FullDuplex(T, T),
}

impl<T> TransmissionMode<T> {
    pub fn is_half_duplex(&self) -> bool {
        match self {
            Self::HalfDuplex(_) => true,
            _ => false,
        }
    }
    pub fn is_full_duplex(&self) -> bool {
        match self {
            Self::FullDuplex(_, _) => true,
            _ => false,
        }
    }
}

impl ClientType {
    pub fn sender() -> SenderAddr {
        let addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        SenderAddr {
            broadcast_addr: SocketAddr::new(IpAddr::V6(addr), 0),
        }
    }
    pub fn receiver() -> ReceiverAddr {
        let default_addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        ReceiverAddr {
            tcp_listener_addr: SocketAddr::new(IpAddr::V6(default_addr), 0),
            udp_socket_addr: SocketAddr::new(IpAddr::V6(default_addr), 0),
            broadcast_addr: SocketAddr::new(IpAddr::V6(default_addr), 0),
        }
    }
    pub fn connect(self) -> io::Result<TransmissionMode<TcpStream>> {
        match self {
            Self::Sender(addr) => {
                let addr = recv_info(addr.broadcast_addr, Duration::from_secs(5))?;
                println!("{:#?}", addr);
                let mut stream = TcpStream::connect(addr.addr)?;
                verify_from_sender(&mut stream)?;
                return sender_transmission_mode(stream);
            }
            Self::Receiver(addr) => {
                let listener = TcpListener::bind(addr.tcp_listener_addr)?;
                let listener_port = listener.local_addr()?.port();
                let is_running = Arc::new(Mutex::new(true));
                let is_running1 = is_running.clone();
                let handler = thread::spawn(move || {
                    send_info(
                        listener_port,
                        is_running1,
                        addr.udp_socket_addr,
                        addr.broadcast_addr,
                    )
                });
                for stream in listener.incoming() {
                    let mut stream = stream?;
                    if let Err(_) = verify_from_receiver(&mut stream) {
                        continue;
                    }
                    close_upd_thread(is_running.clone());
                    println!("Sender Addr: {}", stream.local_addr().unwrap());
                    handler.join().unwrap().unwrap();
                    return receiver_transmission_mode(stream);
                }
                unreachable!()
            }
        }
    }
}
fn verify_from_sender(_stream: &mut TcpStream) -> io::Result<()> {
    // TODO:
    Ok(())
}
fn verify_from_receiver(_stream: &mut TcpStream) -> io::Result<()> {
    // TODO:
    Ok(())
}

fn sender_transmission_mode(stream: TcpStream) -> io::Result<TransmissionMode<TcpStream>> {
    let full_duplex = [111, 0, 111];
    let half_duplex = [0, 111, 0];
    let mut buf = [0; 3];
    match (stream.try_clone(), stream) {
        (Ok(mut stream1), stream2) => {
            stream1.write_all(&full_duplex)?;
            stream1.read_exact(&mut buf)?;
            if buf == full_duplex {
                Ok(TransmissionMode::FullDuplex(stream1, stream2))
            } else if buf == half_duplex {
                Ok(TransmissionMode::HalfDuplex(stream1))
            } else {
                unreachable!()
            }
        }
        (Err(_), mut stream) => {
            stream.write_all(&half_duplex)?;
            stream.read_exact(&mut buf)?;
            if buf == full_duplex || buf == half_duplex {
                Ok(TransmissionMode::HalfDuplex(stream))
            } else {
                unreachable!()
            }
        }
    }
}

fn receiver_transmission_mode(stream: TcpStream) -> io::Result<TransmissionMode<TcpStream>> {
    let full_duplex = [111, 0, 111];
    let half_duplex = [0, 111, 0];
    let mut buf = [0; 3];
    match (stream.try_clone(), stream) {
        (Ok(mut stream1), stream2) => {
            stream1.read_exact(&mut buf)?;
            stream1.write_all(&full_duplex)?;
            if buf == full_duplex {
                Ok(TransmissionMode::FullDuplex(stream1, stream2))
            } else if buf == half_duplex {
                Ok(TransmissionMode::HalfDuplex(stream1))
            } else {
                unreachable!()
            }
        }
        (Err(_), mut stream) => {
            stream.read_exact(&mut buf)?;
            stream.write_all(&half_duplex)?;
            if buf == full_duplex || buf == half_duplex {
                Ok(TransmissionMode::HalfDuplex(stream))
            } else {
                unreachable!()
            }
        }
    }
}

fn send_info(
    port: u16,
    is_running: Arc<Mutex<bool>>,
    socket_addr: SocketAddr,
    broadcast_addr: SocketAddr,
) -> io::Result<()> {
    let socket = UdpSocket::bind(socket_addr)?;
    println!("Udp socket thread sterted"); // TODO: remove me
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;
    let data = UdpDataStream::from(port).to_be_bytes_vec();
    while *is_running.lock().unwrap() {
        socket.send_to(&data, broadcast_addr)?;
        thread::sleep(Duration::from_millis(333));
    }
    println!("Udp socket thread closed"); // TODO: remove me
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClientInfo {
    api_version: u16,
    id: u64,
    os: String,
    user_name: String,
    addr: SocketAddr,
}

fn recv_info(addr: SocketAddr, time_out: Duration) -> io::Result<ClientInfo> {
    let socket = UdpSocket::bind(addr)?;
    socket.set_read_timeout(Some(time_out))?;
    // TODO: use heap allocated buffer if needed
    let mut buf = [0; 2048];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let recv = &buf[0..n];
                let data = UdpDataStream::from_be_bytes(recv);
                if data.is_none() {
                    continue;
                }
                let UdpDataStream {
                    api_version,
                    port,
                    id,
                    os,
                    user_name,
                } = data.unwrap();
                let addr = SocketAddr::new(addr.ip(), port);
                return Ok(ClientInfo {
                    api_version,
                    id,
                    os,
                    user_name,
                    addr,
                });
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}

fn close_upd_thread(is_running: Arc<Mutex<bool>>) {
    let mut is_running = is_running.lock().unwrap();
    *is_running = false;
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use crate::{recv_info, send_info, UdpDataStream};

    #[test]
    fn test_udp_data_stream_to_bytes_vec() {
        let data = b":fs-share:\x00\x63\x1F\x90\x00\x00\x00\x00\x07\x5B\xCD\x15\x00\x05linux\x00\x09eagle1234";
        let want = UdpDataStream {
            api_version: 99,        // \x00\x63
            port: 8080,             // \x1F\x90
            id: 123456789,          // \x00\x00\x00\x00\x07\x5B\xCD\x15
            os: "linux".to_owned(), // \x00\x09linux
            //                         |------| => 16 bit length of str 'linux'
            user_name: "eagle1234".to_owned(), // \x00\x09eagle1234
                                               // |------|  => 16 bit length of str 'eagle1234'
        };
        assert_eq!(data.to_vec(), want.to_be_bytes_vec());
    }
    #[test]
    fn test_udp_data_stream_valid_from_be_bytes() {
        let data = b":fs-share:\x00\x63\x1F\x90\x00\x00\x00\x00\x07\x5B\xCD\x15\x00\x05linux\x00\x09eagle1234";
        let got = UdpDataStream::from_be_bytes(data).unwrap();
        let want = UdpDataStream {
            api_version: 99,
            port: 8080,
            id: 123456789,
            os: "linux".to_owned(),
            user_name: "eagle1234".to_owned(),
        };
        assert_eq!(got, want);
    }
    #[test]
    fn test_udp_data_stream_invalid_from_be_bytes() {
        let data = b":fs-share:\x00\x63\x1F\x90\x00\x00\x00\x00\x07\x5B\xCD\x15\x00\x05linux\x00\x0Aeagle1234";
        let got = UdpDataStream::from_be_bytes(data);
        assert!(got.is_none());
        let data = b":fs-sharE:\x00\x63\x1F\x90\x00\x00\x00\x00\x07\x5B\xCD\x15\x00\x05linux\x00\x09eagle1234";
        let got = UdpDataStream::from_be_bytes(data);
        assert!(got.is_none());
    }
    #[test]
    fn test_send_info() {
        let port = 8080;
        let is_running = Arc::new(Mutex::new(true));
        let is_running_cloned = is_running.clone();
        let broadcast_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7766);
        let handler = thread::spawn(move || {
            send_info(
                port,
                is_running_cloned,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
                broadcast_addr,
            )
            .unwrap();
        });
        // wait a while to start above thread
        thread::sleep(Duration::from_millis(200));
        let info = recv_info(broadcast_addr, Duration::from_secs(1)).unwrap();
        // stop the thread
        *is_running.lock().unwrap() = false;
        handler.join().unwrap();
        assert_eq!(
            info.addr,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
        );
    }
}
