use share_utils::{ReceiverFs, SenderFs};
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    thread,
    time::Duration,
};

#[test]
fn test_0() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "12345678";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut sender = SenderFs::default();
        sender.set_password("12345678".into());
        let addr = SocketAddr::new(addr.ip(), port);
        let r = sender.connect(addr);
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r);
        let mut stream = sender.get_stream().unwrap();
        stream.write_all(b"hello world").unwrap();
    });
    let mut bf = [0; 11];
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"hello world";
    assert_eq!(&bf, want);
}

#[test]
fn test_1() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "12345678";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let mut sender = SenderFs::default();
        sender.set_password("12345679".into()); // wrong password
        sender.connect(addr).unwrap();
    });
    let n = receiver.connect_sender(1).unwrap();
    assert!(n.is_none());
}

#[test]
fn test_2() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "12345678";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut sender = SenderFs::default();
        sender.set_password("12345678".into());
        let addr = SocketAddr::new(addr.ip(), port);
        let r = sender.connect(addr);
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r);
        let mut stream = sender.get_stream().unwrap();
        stream.write_all(b"hello123456789 foooooooo").unwrap();
    });
    let mut bf = [0; 11];
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"hello world";
    assert_eq!(&bf[..5], &want[..5]);
    assert_ne!(&bf[5..], &want[5..]);
}

#[test]
fn test_3() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "password";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut sender = SenderFs::default();
        sender.set_password("password".into());
        let addr = SocketAddr::new(addr.ip(), port);
        let r = sender.connect(addr);
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r);
        let mut stream = sender.get_stream().unwrap();
        stream.write_all(b"hello world12345").unwrap();
    });
    let mut bf = [0; 11];
    let mut n = receiver.connect_sender(9).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"hello world";
    assert_eq!(&bf, want);
}

#[test]
fn test_4() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "strong_password";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    // user woth wrong passwort
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut sender = SenderFs::default();
        sender.set_password("12345679".into()); // wrong password
        let addr = SocketAddr::new(addr.ip(), port);
        sender.connect(addr).unwrap();
    });
    // user with currect password
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        let mut sender = SenderFs::default();
        sender.set_password("strong_password".into());
        let addr = SocketAddr::new(addr.ip(), port);
        sender.connect(addr).unwrap();
    });
    let n = receiver.connect_sender(3).unwrap();
    assert!(n.is_some());
}

#[test]
fn tes_5() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "strong_password";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut sender = SenderFs::default();
        sender.set_password("12345679".into()); // wrong password
        let addr = SocketAddr::new(addr.ip(), port);
        sender.connect(addr).unwrap();
    });
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        let mut sender = SenderFs::default();
        sender.set_password("strong_password".into());
        let addr = SocketAddr::new(addr.ip(), port);
        sender.connect(addr).unwrap();
    });
    let n = receiver.connect_sender(1).unwrap();
    assert!(n.is_none());
}

#[test]
fn test_6() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "secret";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(700));
        let mut sender = SenderFs::default();
        sender.set_password("secret".into());
        let addr = SocketAddr::new(addr.ip(), port);
        let r = sender.connect(addr);
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r);
        let mut stream = sender.get_stream().unwrap();
        stream.write_all(b"123456789123456789").unwrap();
    });
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut sender = SenderFs::default();
        sender.set_password("secret".into());
        let addr = SocketAddr::new(addr.ip(), port);
        let r = sender.connect(addr);
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r);
        let mut stream = sender.get_stream().unwrap();
        stream.write_all(b"abcdefghijklmnopqrstuvwxyz").unwrap();
    });
    thread::sleep(Duration::from_millis(800));
    let mut bf = [0; 11];
    let mut n = receiver.connect_sender(9).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"abcdefghijk";
    assert_eq!(&bf[..], &want[..]);
}

#[test]
fn test_7() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "secret123";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let _ = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
    });
    let n = receiver.connect_sender(1);
    assert!(n.is_ok());
    assert!(n.unwrap().is_none());
}

#[test]
fn test_8() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "secret123";
    let mut receiver = ReceiverFs::default();
    receiver.set_password(password.into());
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream.write_all(b"00").unwrap();
    });
    let n = receiver.connect_sender(1);
    assert!(n.is_ok());
    assert!(n.unwrap().is_none());
}

#[test]
fn test_9() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream.write_all(b"00").unwrap();
    });
    let n = receiver.connect_sender(1);
    assert!(n.is_ok());
    assert!(n.unwrap().is_none());
}

#[test]
fn test_10() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream
            .write_all(b"00000000000000000000000000000000") // 32 zero
            .unwrap();
    });
    let n = receiver.connect_sender(1);
    assert!(n.is_ok());
    assert!(n.unwrap().is_some());
}

#[test]
fn test_11() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream
            .write_all(b"11111111111111111111111111111111") // 32 one
            .unwrap();
    });
    let n = receiver.connect_sender(1);
    assert!(n.is_ok());
    assert!(n.unwrap().is_some());
}

#[test]
fn test_12() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream
            .write_all(b"22222222222222222222222222222299") // 30 two, 2 nine
            .unwrap();
    });
    let n = receiver.connect_sender(1);
    assert!(n.is_ok());
    assert!(n.unwrap().is_some());
}

#[test]
fn test_13() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream
            .write_all(b"1111111111111111111111111111111") // 31 one
            .unwrap();
    });
    let n = receiver.connect_sender(1);
    assert!(n.is_ok());
    assert!(n.unwrap().is_none());
}

#[test]
fn test_14() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let mut receiver = ReceiverFs::default();
    assert!(receiver.bind(addr).is_ok());
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream
            .write_all(b"111111111111111111111111111111119876543210abcdefghijk") // 32 one, ....
            .unwrap();
    });
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    let mut buffer = [0; 10];
    n.read_exact(&mut buffer).unwrap();
    let want = b"9876543210";
    assert_eq!(&buffer, want);
}
