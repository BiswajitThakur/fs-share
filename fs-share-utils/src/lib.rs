mod addr;
mod utils;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub use addr::{get_receiver_addr, get_sender_addr};
use std::fmt::Write as FWrite;
use std::io::{BufWriter, Cursor};
use std::{cmp, fs};
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
#[cfg(test)]
mod tests {
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
