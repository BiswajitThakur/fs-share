use std::{
    fmt::Debug,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    time::Duration,
};

use anyhow::Context;

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
