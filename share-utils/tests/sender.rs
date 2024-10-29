use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
    time::Duration,
};

use share_utils::{ReceiverFs, SenderFs};

#[test]
fn test_0() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let receiver = ReceiverFs::default()
        .set_password("password".into())
        .bind(addr)
        .unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
    assert!(SenderFs::default()
        .set_password("password".into())
        .connect(addr)
        .unwrap()
        .is_connected());
}

#[test]
fn test_1() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let receiver = ReceiverFs::default()
        .set_password("password".into())
        .bind(addr)
        .unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
    assert!(!SenderFs::default()
        .set_password("12345678".into())
        .connect(addr)
        .unwrap()
        .is_connected());
}

#[test]
fn test_2() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let receiver = ReceiverFs::default()
        .set_password("password".into())
        .bind(addr)
        .unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
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
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let receiver = ReceiverFs::default()
        .set_password("password".into())
        .bind(addr)
        .unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(1).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
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
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let receiver = ReceiverFs::default()
        .set_password("123456789".into())
        .bind(addr)
        .unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
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
