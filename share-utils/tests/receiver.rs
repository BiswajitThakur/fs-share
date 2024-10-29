use share_utils::{ReceiverFs, SenderFs, SenderOps};
use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    path::Path,
    thread,
    time::Duration,
};

#[test]
fn test_0() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "12345678";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let sender = SenderFs::default()
            .set_password("12345678".into())
            .connect(addr)
            .unwrap();
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
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        SenderFs::default()
            .set_password("12345679".into()) // wrong password
            .connect(addr)
            .unwrap();
    });
    let n = receiver.connect_sender(1).unwrap();
    assert!(n.is_none());
}

#[test]
fn test_2() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "12345678";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let sender = SenderFs::default()
            .set_password("12345678".into())
            .connect(addr)
            .unwrap();
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
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let sender = SenderFs::default()
            .set_password("password".into())
            .connect(addr)
            .unwrap();
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
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    // user woth wrong passwort
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        SenderFs::default()
            .set_password("12345679".into())
            .connect(addr)
            .unwrap();
    });
    // user with currect password
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        let addr = SocketAddr::new(addr.ip(), port);
        SenderFs::default()
            .set_password("strong_password".into())
            .connect(addr)
            .unwrap();
    });
    let n = receiver.connect_sender(3).unwrap();
    assert!(n.is_some());
}

#[test]
fn tes_5() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "strong_password";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let addr = SocketAddr::new(addr.ip(), port);
        SenderFs::default()
            .set_password("12345679".into()) // wrong password
            .connect(addr)
            .unwrap();
    });
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        let addr = SocketAddr::new(addr.ip(), port);
        SenderFs::default()
            .set_password("strong_password".into())
            .connect(addr)
            .unwrap();
    });
    let n = receiver.connect_sender(1).unwrap();
    assert!(n.is_none());
}

#[test]
fn test_6() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "secret";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(700));
        let addr = SocketAddr::new(addr.ip(), port);
        let sender = SenderFs::default()
            .set_password("secret".into())
            .connect(addr)
            .unwrap();
        let mut stream = sender.get_stream().unwrap();
        stream.write_all(b"123456789123456789").unwrap();
    });
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let sender = SenderFs::default()
            .set_password("secret".into())
            .connect(addr)
            .unwrap();
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
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
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
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
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
    let receiver = ReceiverFs::default().bind(addr).unwrap();
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
    let receiver = ReceiverFs::default().bind(addr).unwrap();
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
    let receiver = ReceiverFs::default().bind(addr).unwrap();
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
    let receiver = ReceiverFs::default().bind(addr).unwrap();
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
    let receiver = ReceiverFs::default().bind(addr).unwrap();
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
    let receiver = ReceiverFs::default().bind(addr).unwrap();
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

#[test]
fn test_15() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let receiver = ReceiverFs::default().bind(addr).unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let mut stream = TcpStream::connect(SocketAddr::new(addr.ip(), port)).unwrap();
        stream.write_all(b"11111111111111").unwrap(); // 14 one
        thread::sleep(Duration::from_millis(300));
        stream.write_all(b"11111111111111111198").unwrap(); // 18 one
        thread::sleep(Duration::from_millis(300));
        stream.write_all(b"76543210abcdefghijk").unwrap();
    });
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    let mut buffer = [0; 10];
    n.read_exact(&mut buffer).unwrap();
    let want = b"9876543210";
    assert_eq!(&buffer, want);
}

#[test]
fn test_16() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let receiver = ReceiverFs::default().bind(addr).unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let sender = SenderFs::default()
            .set_password("password".into())
            .connect(addr)
            .unwrap();
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
fn test_17() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "12345678";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let mut sender = SenderFs::default()
            .set_password("12345678".into())
            .connect(addr)
            .unwrap();
        let r = sender.send(SenderOps::Msg("hello".into())).unwrap();
        assert!(r);
    });
    let mut bf = [0; 11];
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"sm:5:hello:";
    assert_eq!(&bf[..], want);
}

#[test]
fn test_18() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "abcd";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let mut sender = SenderFs::default()
            .set_password("abcd".into())
            .connect(addr)
            .unwrap();
        let r = sender.send(SenderOps::Msg("".into())).unwrap();
        assert!(r);
    });
    let mut bf = [0; 6];
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"sm:0::";
    assert_eq!(&bf[..], want);
}

#[test]
fn test_19() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "12345678";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let mut sender = SenderFs::default()
            .set_password("12345678".into())
            .connect(addr)
            .unwrap();
        sender.send(SenderOps::Msg("hello".into())).unwrap();
        sender.send(SenderOps::Msg("world".into())).unwrap();
        sender
            .send(SenderOps::UserInfo {
                user: Some("Eagle:BT".into()),
            })
            .unwrap()
    });
    let mut bf = [0; 11];
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"sm:5:hello:";
    assert_eq!(&bf[..], want);
    n.read_exact(&mut bf).unwrap();
    let want = b"sm:5:world:";
    assert_eq!(&bf[..], want);
    let want = b"su:8:Eagle:BT:";
    let mut bf = [0; 14];
    n.read_exact(&mut bf).unwrap();
    assert_eq!(&bf[..], want);
}

#[test]
fn test_20() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let password = "abcd";
    let receiver = ReceiverFs::default()
        .set_password(password.into())
        .bind(addr)
        .unwrap();
    let port = receiver.receiver_port().unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        let addr = SocketAddr::new(addr.ip(), port);
        let mut sender = SenderFs::default()
            .set_password("abcd".into())
            .connect(addr)
            .unwrap();
        assert!(sender.send(SenderOps::Msg("It's me Xy".into())).unwrap());
        let v: Vec<u8> = b"abc?@123456789xyz^7**<?>".into();
        let n = v.len();
        let cursor = Cursor::new(v);
        let rdr = BufReader::new(cursor);
        sender
            .send(SenderOps::File {
                name: Path::new("xy-file.txt").into(),
                len: n,
                reader: Box::new(rdr),
            })
            .unwrap();
        assert!(sender.send(SenderOps::Msg("done".into())).unwrap());
    });
    let mut bf = [0; 73];
    let mut n = receiver.connect_sender(1).unwrap().unwrap();
    n.read_exact(&mut bf).unwrap();
    let want = b"sm:10:It's me Xy:sf:11:24:xy-file.txt:abc?@123456789xyz^7**<?>:sm:4:done:";
    assert_eq!(&bf[..], want);
}
