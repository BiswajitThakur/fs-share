use share_utils::{ReceiverFs, SenderFs};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    sync::mpsc,
    thread,
};

const ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

#[test]
fn test_0() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let listener = TcpListener::bind(ADDR).unwrap();
        tx.send(listener.local_addr().unwrap()).unwrap();
        ReceiverFs::default()
            .set_password("password".into())
            .connect_sender(listener, 999)
            .unwrap();
    });
    assert!(SenderFs::default()
        .set_password("password".into())
        .connect(rx.recv().unwrap())
        .unwrap()
        .is_connected());
}

#[test]
fn test_1() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let listener = TcpListener::bind(ADDR).unwrap();
        tx.send(listener.local_addr().unwrap()).unwrap();
        ReceiverFs::default()
            .set_password("password".into())
            .connect_sender(listener, 999)
            .unwrap();
    });
    assert!(!SenderFs::default()
        .set_password("12345678".into())
        .connect(rx.recv().unwrap())
        .unwrap()
        .is_connected());
}

#[test]
fn test_2() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let listener = TcpListener::bind(ADDR).unwrap();
        tx.send(listener.local_addr().unwrap()).unwrap();
        ReceiverFs::default()
            .set_password("password".into())
            .connect_sender(listener, 999)
            .unwrap();
    });
    let addr = rx.recv().unwrap();
    assert!(!SenderFs::default()
        .set_password("12345678".into())
        .connect(addr)
        .unwrap()
        .is_connected());
    assert!(SenderFs::default()
        .set_password("password".into())
        .connect(addr)
        .unwrap()
        .is_connected());
}

#[test]
fn test_3() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let listener = TcpListener::bind(ADDR).unwrap();
        tx.send(listener.local_addr().unwrap()).unwrap();
        ReceiverFs::default()
            .set_password("password".into())
            .connect_sender(listener, 1)
            .unwrap();
    });
    let addr = rx.recv().unwrap();
    assert!(!SenderFs::default()
        .set_password("12345678".into())
        .connect(addr)
        .unwrap()
        .is_connected());
    assert!(SenderFs::default()
        .set_password("password".into())
        .connect(addr)
        .is_err());
}

#[test]
fn test_4() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let listener = TcpListener::bind(ADDR).unwrap();
        tx.send(listener.local_addr().unwrap()).unwrap();
        ReceiverFs::default()
            .set_password("123456789".into())
            .connect_sender(listener, 999)
            .unwrap();
    });
    let addr = rx.recv().unwrap();
    assert!(SenderFs::default()
        .set_password("123456789".into())
        .connect(addr)
        .unwrap()
        .is_connected());
    assert!(SenderFs::default()
        .set_password("123456789".into())
        .connect(addr)
        .is_err())
}
