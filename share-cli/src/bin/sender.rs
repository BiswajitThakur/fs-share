use std::{
    io::{self, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use share_utils::get_receiver_addr;

fn main() {
    if let Ok(addr) = get_receiver_addr("Eagle BT", b"password") {
        println!("Reaciever addr: {:#?}", addr);
        let a = addr.sender;

        std::thread::sleep(Duration::from_secs(2));
        sender(addr.receiver).unwrap();
    }
}

fn sender(addr: SocketAddr) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;

    println!("connected");
    stream.write_all(b"00500000000000000000005a.txthello")?;
    Ok(())
}
