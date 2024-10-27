use std::{
    io::{self, Write},
    net::{SocketAddr, TcpStream},
};

use share_utils::get_receiver_addr;

fn main() {
    if let Ok(addr) = get_receiver_addr("Eagle BT", b"password") {
        println!("Reaciever addr: {:#?}", addr);
        sender(addr.receiver).unwrap();
    }
}

fn sender(addr: SocketAddr) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    println!("connected");
    for line in std::io::stdin().lines() {
        stream.write_all(line?.as_bytes())?;
    }
    Ok(())
}
