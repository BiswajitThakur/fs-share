use std::{
    fmt::Debug,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    time::Duration,
};

use anyhow::Context;
use socket2::{Domain, Socket, Type};

pub fn select_ip() -> Option<IpAddr> {
    select_ip_impl()
}

#[cfg(all(unix, not(target_os = "android")))]
pub fn select_ip_impl() -> Option<IpAddr> {
    use fs_share_utils::ip::IterIpAddr;
    use std::io::Write;

    let mut stdout = std::io::stdout();
    let ips = IterIpAddr::new().unwrap().collect::<Vec<_>>();

    writeln!(&mut stdout, "IP Address List").unwrap();
    for (i, (j, k)) in ips.iter().enumerate() {
        writeln!(&mut stdout, "{}: {} {}", i + 1, j, k).unwrap();
    }
    write!(&mut stdout, "Select Ip Addr: ").unwrap();
    stdout.flush().unwrap();
    let index: usize = get_user_input();
    println!("----------------");
    ips.get(index - 1).map(|(_, ip)| *ip)
}

#[cfg(any(not(unix), target_os = "android"))]
pub fn select_ip_impl() -> Option<IpAddr> {
    None
}

fn get_user_input<T: FromStr>() -> T
where
    <T as FromStr>::Err: Debug,
{
    let stdin = std::io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    input.trim().parse::<T>().expect("Invalid Input")
}

/*
pub fn create_tcp_listener(
    addr: SocketAddr,
) -> anyhow::Result<(SocketAddr, impl Iterator<Item = std::io::Result<TcpStream>>)> {
    struct Dummy {
        inner: TcpListener,
    }

    impl Iterator for Dummy {
        type Item = std::io::Result<TcpStream>;
        fn next(&mut self) -> Option<Self::Item> {
            Some(self.inner.accept().map(|(stream, _)| stream))
        }
    }

    let listener = TcpListener::bind(addr)
        .with_context(|| format!("Failed to bind TCP listener on {}", addr))?;

    let listener_addr = listener
        .local_addr()
        .context("Failed to get local address of TCP listener")?;
    println!("TcpListener Addr: {}", listener_addr);

    Ok((listener_addr, Dummy { inner: listener }))
}
*/

pub fn create_tcp_listener(
    addr: SocketAddr,
) -> anyhow::Result<(SocketAddr, impl Iterator<Item = std::io::Result<TcpStream>>)> {
    struct Dummy {
        inner: TcpListener,
    }
    impl Iterator for Dummy {
        type Item = std::io::Result<TcpStream>;
        fn next(&mut self) -> Option<Self::Item> {
            Some(self.inner.accept().map(|(stream, _)| {
                stream
                    .set_nodelay(true)
                    .expect("Faild to set nodelay: true");
                stream
            }))
        }
    }

    let domain = if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::IPV4
    };
    let socket = Socket::new(domain, Type::STREAM, None).context("Failed to create socket")?;

    socket
        .set_recv_buffer_size(256 * 1024)
        .context("Failed to set recv buffer size")?;
    socket
        .set_send_buffer_size(256 * 1024)
        .context("Failed to set send buffer size")?;

    socket
        .bind(&addr.into())
        .with_context(|| format!("Failed to bind TCP listener on {}", addr))?;

    let listener: TcpListener = socket.into();
    let listener_addr = listener
        .local_addr()
        .context("Failed to get local address of TCP listener")?;

    println!("TcpListener Addr: {}", listener_addr);
    Ok((listener_addr, Dummy { inner: listener }))
}

pub fn receiver_upgrade_stream(mut stream: TcpStream) -> anyhow::Result<TcpStream> {
    let addr = stream.local_addr()?;
    stream
        .set_read_timeout(Some(Duration::from_millis(300)))
        .with_context(|| format!("Faild to set read timeout on {}", addr))?;
    if !match_bytes("v1.fs-share", &mut stream)
        .context("Failed to read protocol header from sender")?
    {
        anyhow::bail!(
            "Protocol mismatch from peer {}: expected 'v1.fs-share' header",
            stream.peer_addr().unwrap_or(addr)
        );
    }
    stream
        .write_all(b":accept:")
        .context("Failed to send upgrade acknowledgement")?;
    stream.flush().context("Failed to flush stream")?;
    stream.set_read_timeout(None)?;
    stream.set_write_timeout(None)?;
    Ok(stream)
}

pub fn sender_upgrade_stream(mut stream: TcpStream) -> anyhow::Result<TcpStream> {
    let addr = stream.local_addr()?;
    stream
        .write_all(b"v1.fs-share")
        .context("Failed to send protocol header")?;
    stream.flush().context("Failed to flush stream")?;
    if !match_bytes(":accept:", &mut stream)
        .context("Failed to read upgrade acknowledgement from receiver")?
    {
        anyhow::bail!(
            "Protocol upgrade rejected by peer {}: expected ':accept:' response",
            stream.peer_addr().unwrap_or(addr)
        );
    };
    stream.set_read_timeout(None)?;
    stream.set_write_timeout(None)?;
    Ok(stream)
}

fn match_bytes<B: AsRef<[u8]>, R: Read>(bytes: B, mut reader: R) -> anyhow::Result<bool> {
    let expected = bytes.as_ref();

    let mut buf = vec![0u8; expected.len()].into_boxed_slice();

    reader
        .read_exact(&mut buf)
        .context("Failed to read bytes from reader")?;

    Ok(buf.as_ref() == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    #[test]
    fn match_bytes_returns_true_on_exact_match() {
        let data = b"v1.fs-share";
        let cursor = Cursor::new(data);
        assert!(match_bytes("v1.fs-share", cursor).unwrap());
    }

    #[test]
    fn match_bytes_returns_false_on_mismatch() {
        let data = b"v2.fs-share";
        let cursor = Cursor::new(data);
        assert!(!match_bytes("v1.fs-share", cursor).unwrap());
    }

    #[test]
    fn match_bytes_returns_false_on_partial_match() {
        let data = b"v1.fs-sha";
        let cursor = Cursor::new(data);
        assert!(match_bytes("v1.fs-share", cursor).is_err());
    }

    #[test]
    fn match_bytes_returns_true_for_accept_token() {
        let cursor = Cursor::new(b":accept:");
        assert!(match_bytes(":accept:", cursor).unwrap());
    }

    #[test]
    fn match_bytes_errors_on_empty_reader() {
        let cursor = Cursor::new(b"");
        assert!(match_bytes("v1.fs-share", cursor).is_err());
    }

    #[test]
    fn match_bytes_works_with_byte_slice_input() {
        let cursor = Cursor::new(b"hello");
        assert!(match_bytes(b"hello", cursor).unwrap());
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Binds a random port and returns (listener, local_addr).
    fn make_listener() -> (TcpListener, std::net::SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        (listener, addr)
    }

    /// Spawns a thread that accepts one connection and runs `f` on it,
    /// returning a JoinHandle so the test can inspect the result.
    fn accept_with<F, T>(listener: TcpListener, f: F) -> thread::JoinHandle<anyhow::Result<T>>
    where
        F: FnOnce(TcpStream) -> anyhow::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            f(stream)
        })
    }

    // ── sender_upgrade_stream / receiver_upgrade_stream ───────────────────────

    #[test]
    fn upgrade_handshake_succeeds() {
        let (listener, addr) = make_listener();

        let receiver_handle = accept_with(listener, receiver_upgrade_stream);

        let sender_stream = TcpStream::connect(addr).unwrap();
        let sender_result = sender_upgrade_stream(sender_stream);

        let receiver_result = receiver_handle.join().unwrap();

        assert!(
            sender_result.is_ok(),
            "sender upgrade failed: {:?}",
            sender_result
        );
        assert!(
            receiver_result.is_ok(),
            "receiver upgrade failed: {:?}",
            receiver_result
        );
    }

    #[test]
    fn receiver_rejects_wrong_protocol_header() {
        let (listener, addr) = make_listener();

        let receiver_handle = accept_with(listener, receiver_upgrade_stream);

        // Send the wrong header instead of "v1.fs-share"
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(b"v2.fs-share").unwrap();
            stream.flush().unwrap();
        });

        let result = receiver_handle.join().unwrap();
        assert!(result.is_err(), "expected error for wrong header");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("Protocol mismatch"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn sender_rejects_wrong_acknowledgement() {
        let (listener, addr) = make_listener();

        // Receiver side: reads "v1.fs-share" correctly but sends wrong ack
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; b"v1.fs-share".len()];
            stream.read_exact(&mut buf).unwrap();
            stream.write_all(b":reject:").unwrap(); // wrong ack
            stream.flush().unwrap();
        });

        let sender_stream = TcpStream::connect(addr).unwrap();
        let result = sender_upgrade_stream(sender_stream);

        assert!(result.is_err(), "expected error for wrong ack");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("Protocol upgrade rejected"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn receiver_errors_when_sender_sends_nothing() {
        let (listener, addr) = make_listener();

        let receiver_handle = accept_with(listener, receiver_upgrade_stream);

        // Connect but immediately drop — sends EOF
        let stream = TcpStream::connect(addr).unwrap();
        drop(stream);

        let result = receiver_handle.join().unwrap();
        assert!(result.is_err(), "expected error on empty stream");
    }

    #[test]
    fn sender_errors_when_receiver_sends_nothing() {
        let (listener, addr) = make_listener();

        // Accept and immediately drop without sending anything back
        thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            // drop here — EOF on sender's read
        });

        let sender_stream = TcpStream::connect(addr).unwrap();
        let result = sender_upgrade_stream(sender_stream);
        assert!(
            result.is_err(),
            "expected error when receiver drops connection"
        );
    }

    #[test]
    fn upgraded_streams_have_no_timeouts() {
        let (listener, addr) = make_listener();

        let receiver_handle = accept_with(listener, receiver_upgrade_stream);

        let sender_stream = TcpStream::connect(addr).unwrap();
        let sender = sender_upgrade_stream(sender_stream).unwrap();
        let receiver = receiver_handle.join().unwrap().unwrap();

        assert!(
            sender.read_timeout().unwrap().is_none(),
            "sender read timeout should be None"
        );
        assert!(
            sender.write_timeout().unwrap().is_none(),
            "sender write timeout should be None"
        );
        assert!(
            receiver.read_timeout().unwrap().is_none(),
            "receiver read timeout should be None"
        );
        assert!(
            receiver.write_timeout().unwrap().is_none(),
            "receiver write timeout should be None"
        );
    }
}
