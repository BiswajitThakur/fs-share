use std::{
    fs,
    io::{self, BufReader},
    net::TcpListener,
    path::PathBuf,
    time::Duration,
};

use clap::{Args, CommandFactory, Parser, Subcommand};
use share_utils::{receiver_addr, sender_addr, ReceiverFs, ReceiverOps, SenderFs, SenderOps};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// your name
    name: Option<String>,

    /// password
    password: Option<String>,

    /// files
    args: Vec<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Send,
    Receive,
}

impl Cli {
    pub fn run(self) -> io::Result<()> {
        match self.command {
            Commands::Send => {
                let name = self.name.unwrap_or_default();
                let password: Vec<u8> = self.password.unwrap_or_default().into();
                let receiver = receiver_addr(name.as_ref(), &password)?.unwrap();
                let (addr, receiver_name) = receiver;
                println!("Receiver name: {}", receiver_name);
                println!("         addr: {}", addr);
                std::thread::sleep(Duration::from_secs(2));
                let mut sender = SenderFs::default()
                    .set_password(password)
                    .connect(addr)
                    .unwrap();
                for arg in self.args {
                    println!("sending file: {}", &arg.display());
                    sender.send(SenderOps::Msg(
                        format!("Sending file: {}", arg.display()).into(),
                    ))?;
                }
            }
            Commands::Receive => {
                let name = self.name.unwrap_or_default();
                let password: Vec<u8> = self.password.unwrap_or_default().into();
                let sender = sender_addr(name.as_ref(), &password)?.unwrap();
                let (addr, sender_name) = sender;
                println!("Sender name: {}", sender_name);
                println!("       addr: {}", addr);
                let mut receiver = ReceiverFs::default()
                    .set_password(password)
                    .connect_sender(TcpListener::bind(addr).unwrap(), 1)
                    .unwrap();
                while let Ok(v) = receiver.receive() {
                    match v {
                        ReceiverOps::Msg(m) => println!("{}", m),
                        ReceiverOps::File { name, size } => {
                            println!("file name: {}, size:{}", name.display(), size)
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
