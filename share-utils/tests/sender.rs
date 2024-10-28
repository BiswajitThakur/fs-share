use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
    time::Duration,
};

use share_utils::{ReceiverFs, SenderFs};

#[test]
fn test_0() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    receiver.set_password("password".into());
    receiver.bind(addr).unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
    let mut sender = SenderFs::default();
    sender.set_password("password".into());
    let r = sender.connect(addr);
    assert!(r.is_ok());
    assert!(r.unwrap());
}

#[test]
fn test_1() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    receiver.set_password("password".into());
    receiver.bind(addr).unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
    let mut sender = SenderFs::default();
    sender.set_password("12345678".into());
    let r = sender.connect(addr);
    let q = sender.get_stream();
    //
    assert!(r.is_ok());
    assert!(!r.unwrap());
}

#[test]
fn test_2() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    receiver.set_password("password".into());
    receiver.bind(addr).unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
    {
        let mut sender = SenderFs::default();
        sender.set_password("12345678".into());
        let r = sender.connect(addr);
        assert!(r.is_ok());
        assert!(!r.unwrap());
    }
    {
        let mut sender = SenderFs::default();
        sender.set_password("password".into());
        let r = sender.connect(addr);
        assert!(r.is_ok());
        assert!(r.unwrap());
    }
}

#[test]
fn test_3() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    receiver.set_password("password".into());
    receiver.bind(addr).unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(1).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
    {
        let mut sender = SenderFs::default();
        sender.set_password("12345678".into());
        let r = sender.connect(addr);
        assert!(r.is_ok());
        assert!(!r.unwrap());
    }
    {
        let mut sender = SenderFs::default();
        sender.set_password("password".into());
        let r = sender.connect(addr);
        assert!(r.is_err());
    }
}

#[test]
fn test_4() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    receiver.set_password("123456789".into());
    receiver.bind(addr).unwrap();
    let addr = receiver.receiver_addr().unwrap();
    thread::spawn(move || {
        receiver.connect_sender(999).unwrap();
    });
    thread::sleep(Duration::from_millis(300));
    {
        let mut sender = SenderFs::default();
        sender.set_password("123456789".into());
        let r = sender.connect(addr);
        assert!(r.is_ok());
        assert!(r.unwrap());
    }
    {
        let mut sender = SenderFs::default();
        sender.set_password("123456789".into());
        let r = sender.connect(addr);
        assert!(r.is_err());
    }
}
