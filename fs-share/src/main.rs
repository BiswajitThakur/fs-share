use std::{
    cell::OnceCell,
    net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener},
    thread,
};

use share_utils::ClientType;

mod cli;

const PASS: OnceCell<i32> = OnceCell::new();

fn main() -> std::io::Result<()> {
    //cli::run()

    //let listener = TcpListener::bind("[fe80::87a9:3b1f:c96:108a]:0").unwrap();
    //println!("Addr: {}", listener.local_addr().unwrap());
    let option = std::env::args().skip(1).next().unwrap();
    match option.as_str() {
        "send" | "--send" => {
            println!("Send >>>>"); // TODO: remove me
            let client = ClientType::sender()
                .set_broadcast_addr(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3434))
                .build();
            let stream = client.connect()?;
            /*
            let (sender, recv) = stream;
            thread::spawn(move || recv.receive());
            for file in files {
                sender.send(file)?;
            }*/
        }
        "receive" | "--receive" => {
            println!("Recv >>>>"); // TODO: remove me
            let client = ClientType::receiver()
                .set_tcp_listener_addr(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0))
                .set_udp_socket_addr(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0))
                .set_broadcast_addr(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3434))
                .build();
            let stream = client.connect()?;
            /*
            let (sender, recv) = stream;
            thread::spawn(move || recv.receive());
            for file in files {
                sender.send(file)?;
            }*/
        }

        _ => {}
    }
    Ok(())
}
