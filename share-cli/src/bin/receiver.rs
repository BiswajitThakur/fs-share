use std::{
    io::{self, BufRead, BufReader, Write},
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
    let mut stdout = std::io::stdout().lock();
    let rdr = BufReader::new(&mut stream);
    for line in rdr.lines() {
        stdout.write_all(line?.as_bytes())?;
    }
    Ok(())
}
