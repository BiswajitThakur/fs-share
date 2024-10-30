use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

use share_utils::{get_sender_addr, ReceiverFs, ReceiverOps};

fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    //if let Ok(addr) = get_sender_addr("", b"password") {
    //   println!("Sender addr: {:#?}", addr);
    let mut receiver = ReceiverFs::default()
        .set_password("12345678".into())
        //.connect_sender(TcpListener::bind(addr.receiver).unwrap(), 1)
        .connect_sender(TcpListener::bind(addr).unwrap(), 1)
        .unwrap();
    while let Ok(v) = receiver.receive() {
        match v {
            ReceiverOps::Msg(m) => println!("{}", m),
            ReceiverOps::File { name, size } => {
                println!("file name: {}, size:{}", name.display(), size)
            }
            _ => {}
        }
        println!("ssss");
    }
    // }
}
