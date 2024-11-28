mod addr;
mod utils;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub use addr::{get_receiver_addr, get_sender_addr};
use std::fmt::Write as FWrite;
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    net::TcpStream,
    path::Path,
};
pub use utils::sha256;

pub trait ShareFs {
    fn send_empty(&mut self) -> io::Result<()>;
    fn receive(&mut self) -> io::Result<bool>;
    fn send_msg<T: AsRef<str>>(&mut self, value: T) -> io::Result<()>;
    fn send_info<T: AsRef<str>>(&mut self, name: T) -> io::Result<()>;
    fn send_file<P: AsRef<Path>>(&mut self, file: P) -> io::Result<()>;
}

impl ShareFs for TcpStream {
    fn send_empty(&mut self) -> io::Result<()> {
        write!(self, ":00:")
    }
    fn send_msg<T: AsRef<str>>(&mut self, value: T) -> io::Result<()> {
        let msg = value.as_ref();
        write!(self, "msg:{}:{}", msg.as_bytes().len(), msg)?;
        let mut v = Vec::with_capacity(4);
        read_n_bytes(self, &mut v, 4)?;
        let prefix = std::str::from_utf8(&v).unwrap_or_default();
        match prefix {
            ":ss:" => println!(".....success....."),
            _ => {
                eprintln!(".....Falid to Send.....");
                std::process::exit(1);
            }
        }
        Ok(())
    }
    fn send_info<T: AsRef<str>>(&mut self, _name: T) -> io::Result<()> {
        todo!()
    }
    fn send_file<P: AsRef<Path>>(&mut self, file: P) -> io::Result<()> {
        let f_name = file
            .as_ref()
            .display()
            .to_string()
            .replace("\"", "")
            .replace("'", "");
        let f = File::open(file)?;
        let file_len = f.metadata()?.len();
        println!("Sending file: {}, size: {} bytes", &f_name, file_len);
        write!(self, "fff:{}:{}:{}:", f_name.len(), f_name, file_len)?;
        let mut reader = BufReader::new(f);
        let mut buffer: Vec<u8> = vec![0; 32 * 1024];
        let mut i = 0;
        let pb = ProgressBar::new(file_len);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn FWrite| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
        loop {
            let r = reader.read(&mut buffer)?;
            if r == 0 {
                break;
            }
            i += r as u64;
            pb.set_position(i);
            self.write_all(&buffer[..r])?;
        }
        self.read_exact(&mut buffer[0..4])?;
        let prefix = std::str::from_utf8(&buffer[0..4]).unwrap_or_default();
        match prefix {
            ":ss:" => pb.finish_with_message(".....success....."),
            _ => {
                pb.finish_with_message(".....Falid to Send.....");
                std::process::exit(1);
            }
        }
        Ok(())
    }
    fn receive(&mut self) -> io::Result<bool> {
        let mut v: Vec<u8> = Vec::with_capacity(4);
        read_n_bytes(self, &mut v, 4)?;
        let prefix = std::str::from_utf8(&v).unwrap_or_default();
        match prefix {
            ":00:" => return Ok(false),
            "msg:" => {
                let len = read_num(self)?;
                let mut stdout = io::stdout().lock();
                read_n_bytes(self, &mut stdout, len)?;
                stdout.write_all(b"\n")?;
                self.write_all(b":ss:")?;
                return Ok(true);
            }
            _ => {}
        }

        todo!()
    }
}

fn read_n_bytes<R: io::Read, W: io::Write>(r: &mut R, w: &mut W, n: usize) -> io::Result<()> {
    let mut remain = n;
    let mut buffer = vec![0; std::cmp::min(n, 1024 * 32)];

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
    pb.finish_and_clear();
    Ok(())
}

fn read_num<R: io::Read>(r: &mut R) -> io::Result<usize> {
    let mut buffer = [0; 1];
    let mut num = 0;
    loop {
        let c = r.read(&mut buffer)?;
        if c == 0 || !matches!(buffer[0], b'0'..=b'9') {
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
