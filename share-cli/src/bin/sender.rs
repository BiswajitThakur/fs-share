use std::{
    fs,
    io::{self, BufReader},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};

use share_utils::{get_receiver_addr, SenderFs, SenderOps};

fn main() -> io::Result<()> {
    let addr = get_receiver_addr("User 1", b"12345678").unwrap();
    println!("Receiver add: {}", addr.receiver);
    std::thread::sleep(Duration::from_secs(2));
    let mut sender = SenderFs::default()
        .set_password("12345678".into())
        .connect(addr.receiver)
        .unwrap();
    for args in std::env::args() {
        let f = fs::File::open(&args)?;
        let len = f.metadata()?.len();
        let rdr = BufReader::new(f);
        println!("sending file: {}", &args);
        sender
            .send(SenderOps::File {
                name: PathBuf::from(args).into(),
                len: len as usize,
                reader: Box::new(rdr),
            })
            .unwrap();
    }
    // if let Ok(addr) = get_receiver_addr("Eagle BT", b"password") {
    //    println!("Reaciever addr: {:#?}", addr);
    /*
        println!("{}", args);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    std::thread::sleep(Duration::from_secs(2));
    let mut sender = SenderFs::default()
        .set_password("12345678".into())
        //.connect(addr.receiver)
        .connect(addr)
        .unwrap();
    for line in std::io::stdin().lines() {
        let line = line.unwrap();
        sender
            .send(SenderOps::Msg(format!("{}", line).into()))
            .unwrap();
        let f = fs::File::open("/home/eagle/movies/American.Psycho.2000.REMASTERED.1080p.BluRay.x264.DD5.1-[Mkvking.com].mkv").unwrap();
        let len = f.metadata().unwrap().len();
        let rdr = BufReader::new(f);
        sender
            .send(SenderOps::File {
                name: Path::new("yy").into(),
                len: len as usize,
                reader: Box::new(rdr),
            })
            .unwrap();
        // sender.read_exact(&mut buf).unwrap();
        // if &buf == b"done" {
        // eprintln!("send success");
        // } else {
        //     eprintln!("faild to send");
        // }
    }
    // }*/
    Ok(())
}
