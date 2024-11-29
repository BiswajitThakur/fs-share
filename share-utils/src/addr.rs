use std::{
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    thread,
    time::{Duration, Instant},
};

use crate::sha256;

pub fn get_receiver_addr<A: ToSocketAddrs, P: AsRef<[u8]>>(
    socket: UdpSocket,
    password: P,
    broadcast_addr: A,
    timeout: Duration,
) -> io::Result<Option<SocketAddr>> {
    // let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;
    //let broadcast_addr = "255.255.255.255:34254";
    let hash = sha256(password);
    let hash_of_hash = sha256(hash.as_slice());
    let mut buf: [u8; 32] = [0; 32];
    let mut data = b":fs-share:".to_vec();
    data.extend(hash_of_hash);
    let now = Instant::now();
    loop {
        if now.elapsed() >= timeout {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(300));
        socket.send_to(&data, &broadcast_addr)?;

        match socket.recv_from(&mut buf) {
            Ok((size, addr)) if size == 32 && buf == hash.as_slice() => {
                socket.send_to(b":success:", addr)?;
                return Ok(Some(addr));
            }
            Err(e) => eprintln!("{e}"),
            _ => {}
        };
    }
}

pub fn get_sender_addr<T: AsRef<[u8]>>(
    socket: UdpSocket,
    password: T,
    timeout: Duration,
) -> io::Result<Option<SocketAddr>> {
    //let socket = UdpSocket::bind("127.0.0.1:0")?;
    let hash = sha256(password.as_ref());
    let hash_of_hash = sha256(&hash);
    let mut buf: [u8; 42] = [0; 42];
    let now = Instant::now();
    loop {
        if now.elapsed() >= timeout {
            return Ok(None);
        }
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) if size == 9 => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                if msg.contains(":success:") {
                    return Ok(Some(addr));
                }
            }
            Ok((42, addr)) => {
                if buf[..10] == *b":fs-share:" && buf[10..] == hash_of_hash {
                    socket.send_to(hash.as_slice(), addr)?;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::UdpSocket,
        thread,
        time::{Duration, Instant},
    };

    use super::{get_receiver_addr, get_sender_addr};
    use crate::sha256;

    #[test]
    fn test_get_receiver_addr_timeout() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = get_receiver_addr(
            UdpSocket::bind("127.0.0.1:0").unwrap(),
            "password",
            socket.local_addr().unwrap(),
            Duration::from_secs(2),
        )
        .unwrap();
        assert!(addr.is_none());
    }

    #[test]
    fn test_get_receiver_addr() {
        let socket1 = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr1 = socket1.local_addr().unwrap();
        let broadcast = UdpSocket::bind("127.0.0.1:0").unwrap();
        let broadcast_addr = broadcast.local_addr().unwrap();
        let password = b"my-password";
        let hash = sha256(password);
        let hash_of_hash = sha256(&hash);
        let timeout = Duration::from_secs(5);
        let start = Instant::now();
        thread::spawn(move || {
            get_receiver_addr(socket1, password, broadcast_addr, timeout).unwrap();
        });
        let mut buf: [u8; 42] = [0; 42];
        loop {
            if start.elapsed() >= timeout {
                panic!("Time Out");
            }
            match broadcast.recv_from(&mut buf) {
                Ok((size, addr)) if size == 42 => {
                    assert_eq!(addr, addr1);
                    assert_eq!(buf[..10], *b":fs-share:");
                    assert_eq!(&buf[10..], &hash_of_hash);
                    break;
                }
                Err(e) => eprintln!("{e}"),
                _ => {}
            }
        }
        let start = Instant::now();
        let mut flag = false;
        loop {
            if start.elapsed() >= timeout {
                panic!("Time Out");
            }
            match broadcast.recv_from(&mut buf) {
                Ok((size, addr)) if size == 42 => {
                    assert_eq!(addr, addr1);
                    assert_eq!(buf[..10], *b":fs-share:");
                    assert_eq!(&buf[10..], &hash_of_hash);
                    broadcast.send_to(&hash, addr).unwrap();
                    flag = true;
                }
                Ok((size, addr)) if size == 9 => {
                    assert_eq!(addr, addr1);
                    assert_eq!(buf[..size], *b":success:");
                    break;
                }
                Err(e) => eprintln!("{e}"),
                _ => {}
            }
        }
        assert!(flag);
    }

    #[test]
    fn test_get_sender_addr() {
        let broadcast = UdpSocket::bind("127.0.0.1:0").unwrap();
        let broadcast_addr = broadcast.local_addr().unwrap();
        thread::spawn(move || get_sender_addr(broadcast, "my-password", Duration::from_secs(5)));
        let r = get_receiver_addr(
            UdpSocket::bind("127.0.0.1:0").unwrap(),
            "password",
            broadcast_addr,
            Duration::from_secs(3),
        );
        assert!(r.unwrap().is_none());
        let r = get_receiver_addr(
            UdpSocket::bind("127.0.0.1:0").unwrap(),
            "my-password",
            broadcast_addr,
            Duration::from_secs(5),
        );
        assert!(r.unwrap().is_some());
    }
}
