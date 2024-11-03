use clap::{Args, Parser, Subcommand};
use share_utils::{
    receiver_addr, sender_addr, Connector, ReceiverFs, ReceiverOps, SenderFs, SenderOps,
};
use std::{
    fs,
    io::{self, BufReader},
    net::TcpListener,
    path::PathBuf,
    time::Duration,
};

/*
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    global: GlobalOptions,
}

#[derive(Parser)]
struct GlobalOptions {
    /// Your name
    #[arg(long, )]
    name: Option<String>,

    /// Password
    #[arg(long)]
    password: Option<String>,

    /// Files
    #[arg()]
    args: Vec<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Send,
    Receive,
}
*/
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Send(SharedOptions),
    Receive(SharedOptions),
}

#[derive(Args)]
struct SharedOptions {
    /// Your name
    #[arg(long)]
    name: Option<String>,

    // /// Password
    // #[arg(long)]
    // password: Option<String>,
    /// Files
    #[arg()]
    args: Vec<PathBuf>,
}
impl Cli {
    pub fn run(self) -> io::Result<()> {
        match &self.command {
            Commands::Send(options) => {
                let name = options.name.clone().unwrap_or("Unknown".into());
                let addr = Connector::new(name, Vec::new())
                    .receiver_addr()
                    .unwrap()
                    .receiver;
                println!("Receiver addr: {}", addr);
                std::thread::sleep(Duration::from_secs(2));
                let mut sender = SenderFs::default()
                    .set_password("12345678".into())
                    .connect(addr)
                    .unwrap();
                for file in &options.args {
                    sender
                        .send(SenderOps::Msg(file.display().to_string().into()))
                        .unwrap();
                }
            }
            Commands::Receive(options) => {
                let name = options.name.clone().unwrap_or("Unknown".into());
                let addr = Connector::new(name, Vec::new())
                    .sender_addr()
                    .unwrap()
                    .receiver;
                let mut receiver = ReceiverFs::default()
                    .set_password("12345678".into())
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
                for file in &options.args {
                    println!("File: {:?}", file);
                }
            }
        }
        /*
        match self.command {
            Commands::Send => {
                let name = self.global.name.unwrap_or_default();
                let password: Vec<u8> = self.global.password.unwrap_or_default().into();
                let receiver = receiver_addr(name.as_ref(), &password)?.unwrap();
                let (addr, receiver_name) = receiver;
                println!("Receiver name: {}", receiver_name);
                println!("         addr: {}", addr);
                std::thread::sleep(Duration::from_secs(2));
                let mut sender = SenderFs::default()
                    .set_password(password)
                    .connect(addr)
                    .unwrap();
                for arg in self.global.args {
                    println!("sending file: {}", &arg.display());
                    sender.send(SenderOps::Msg(
                        format!("Sending file: {}", arg.display()).into(),
                    ))?;
                }
            }
            Commands::Receive => {
                let name = self.global.name.unwrap_or_default();
                let password: Vec<u8> = self.global.password.unwrap_or_default().into();
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
        }*/
        Ok(())
    }
}
