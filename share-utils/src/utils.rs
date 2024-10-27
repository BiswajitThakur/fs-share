use std::{
    io,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use crate::Address;

#[allow(unused)]
pub fn get_receiver_addr(name: &str, password: &[u8]) -> io::Result<Address> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;
    let broadcast_addr = "255.255.255.255:34254";
    let mut buf: [u8; 32] = [0; 32];
    loop {
        socket.send_to(name.as_bytes(), broadcast_addr)?;
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) if size <= 32 && &buf[..size] == password => {
                socket.send_to(format!("{}: success", name).as_bytes(), addr)?;
                return Ok(Address {
                    sender: socket.local_addr()?,
                    receiver: addr,
                });
            }
            _ => {}
        };
    }
}

#[allow(unused)]
pub fn get_sender_addr(_name: &str, password: &[u8]) -> io::Result<Address> {
    let socket = UdpSocket::bind("0.0.0.0:34254")?;
    let mut buf: [u8; 32] = [0; 32];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                if msg.contains("success") {
                    return Ok(Address {
                        sender: addr,
                        receiver: socket.local_addr()?,
                    });
                }
                socket.send_to(password, addr)?;
            }
            _ => {}
        }
    }
}
