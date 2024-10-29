use std::time::Duration;

use share_utils::{get_receiver_addr, SenderFs, SenderOps};

fn main() {
    if let Ok(addr) = get_receiver_addr("Eagle BT", b"password") {
        println!("Reaciever addr: {:#?}", addr);
        std::thread::sleep(Duration::from_secs(2));
        let mut sender = SenderFs::default()
            .set_password("12345678".into())
            .connect(addr.receiver)
            .unwrap();
        for line in std::io::stdin().lines() {
            let line = line.unwrap();
            sender.send(SenderOps::Msg(line.into())).unwrap();
        }
    }
}

/*
fn sender(addr: SocketAddr) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;

    println!("connected");
    stream.write_all(b"00500000000000000000005a.txthello")?;
    Ok(())
}*/
