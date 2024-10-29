use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
};

use share_utils::{get_sender_addr, ReceiverFs};

fn main() {
    if let Ok(addr) = get_sender_addr("", b"password") {
        println!("Sender addr: {:#?}", addr);
        let receiver = ReceiverFs::default()
            .set_password("12345678".into())
            .connect_sender(TcpListener::bind(addr.receiver).unwrap(), 1)
            .unwrap();
        let rdr = BufReader::new(receiver.get_stream().unwrap());
        for line in rdr.lines() {
            println!("{}", line.unwrap());
        }
    }
}
