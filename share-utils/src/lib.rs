mod utils;

use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
};

pub use utils::{get_receiver_addr, get_sender_addr};

#[derive(Debug, Clone)]
pub struct Address {
    pub sender: SocketAddr,
    pub receiver: SocketAddr,
}

pub const PORT: u16 = 34254;

pub fn create_tcp_connection<F: Fn(TcpStream) -> io::Result<()>>(
    addr: Address,
    f: F,
) -> io::Result<()> {
    let listener = TcpListener::bind(addr.receiver)?;
    for stream in listener.incoming() {
        match stream {
            Ok(v) => {
                if v.local_addr()? == addr.sender {
                    f(v)?
                }
            }
            _ => {}
        }
    }
    Ok(())
}
