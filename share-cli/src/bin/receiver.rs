use std::{
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
};

use share_utils::{create_tcp_connection, get_sender_addr};

fn main() {
    if let Ok(addr) = get_sender_addr("", b"password") {
        println!("Sender addr: {:#?}", addr);
        create_tcp_connection(addr, handle_connection).unwrap()
    }
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let mut rdr = BufReader::new(&mut stream);
    let (name, body_len, extra) = read_name_body_len(&mut rdr)?;
    let f = std::fs::File::create_new(name)?;
    let mut bfr = BufWriter::new(f);
    bfr.write_all(&extra)?;
    let mut buffer = [0; 4096]; // 4kb
    let mut r = 0;
    while r < body_len {
        let require = body_len - r;
        let read = rdr.read(&mut buffer)?;
        if require >= read {
            bfr.write_all(&buffer[..read])?;
            r += read;
            continue;
        } else {
            todo!()
        }
    }
    Ok(())
}

fn read_name_body_len(rdr: &mut BufReader<&mut TcpStream>) -> io::Result<(String, usize, Vec<u8>)> {
    let mut nl: [u8; 23] = [0; 23];
    let mut name_len: usize = 0;
    let mut body_len: usize = 0;
    rdr.read_exact(&mut nl)?;
    for c in &nl[..3] {
        if !c.is_ascii_digit() {
            panic!("ERROR: invalid char, not a number...");
        }
        name_len += *c as usize - '0' as usize;
    }
    if name_len > 100 {
        panic!("File name is too long...");
    }
    for c in &nl[3..] {
        body_len += *c as usize - '0' as usize
    }
    let mut name = String::new();

    let mut r = 0;
    while r < 23 {
        let n = rdr.read(&mut nl)?;
        if let Some(require) = 23usize.checked_sub(r) {
            if require >= n {
                for c in &nl[..n] {
                    name.push(*c as char);
                }
                r += n;
                continue;
            } else {
                for c in &nl[..require] {
                    name.push(*c as char);
                }
                let mut extra = Vec::with_capacity(n - 23);
                for u in &nl[require..] {
                    extra.push(*u);
                }
                return Ok((name, body_len, extra));
            }
        } else {
            unreachable!()
        }
    }
    Ok((name, body_len, Vec::new()))
}
