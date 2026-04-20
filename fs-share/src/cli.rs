use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};

const BROADCAST_PORT: u16 = 7755;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Debug, Subcommand)]
pub enum Mode {
    Send {
        #[arg(short, long)]
        receiver_addr: Option<SocketAddr>,

        #[arg(short, long)]
        download_dir: Option<PathBuf>,

        #[arg(long)]
        disable_progress: bool,

        #[arg(long, default_value_t = BROADCAST_PORT)]
        broadcast_port: u16,

        #[arg()]
        args: Vec<PathBuf>,
    },

    Receive {
        #[arg(short, long)]
        tcp_listener_addr: Option<SocketAddr>,

        #[arg(short, long)]
        download_dir: Option<PathBuf>,

        #[arg(long)]
        disable_broadcast: bool,

        #[arg(long)]
        disable_progress: bool,

        #[arg(short, long, default_value_t = BROADCAST_PORT)]
        broadcast_port: u16,

        #[arg()]
        args: Vec<PathBuf>,
    },
}
