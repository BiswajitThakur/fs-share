use std::{net::UdpSocket, thread, time::Duration};

use share_utils::Addr;

#[test]
fn test_0() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = socket.local_addr().unwrap();
    thread::spawn(move || {
        let _r = Addr::default()
            .set_broadcast_addr(addr)
            .receiver_addr(socket)
            .unwrap();
    });
    thread::sleep(Duration::from_secs(1));
    let _sender = Addr::default().sender_addr().unwrap();
}
