use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};

/// Default UDP broadcast port used for discovery
const BROADCAST_PORT: u16 = 7755;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Select application mode (send or receive)
    #[command(subcommand)]
    pub mode: Mode,
}

/// Available CLI modes
#[derive(Debug, Subcommand)]
pub enum Mode {
    /// Sender mode
    Send {
        /// Manually specify receiver address (skip auto-discovery)
        #[arg(short, long)]
        receiver_addr: Option<SocketAddr>,

        /// Directory where received files will be saved
        #[arg(short, long)]
        download_dir: Option<PathBuf>,

        /// Disable progress bar output
        #[arg(long)]
        disable_progress: bool,

        /// UDP broadcast port for discovering receivers
        #[arg(long, default_value_t = BROADCAST_PORT)]
        broadcast_port: u16,

        /// Files to send
        #[arg()]
        args: Vec<PathBuf>,
    },

    /// Receiver mode
    Receive {
        /// TCP listener address (IP:PORT) for incoming connections
        #[arg(short, long)]
        tcp_listener_addr: Option<SocketAddr>,

        /// Directory to save received files
        #[arg(short, long)]
        download_dir: Option<PathBuf>,

        /// Disable broadcasting presence (no auto-discovery)
        #[arg(long)]
        disable_broadcast: bool,

        /// Disable progress bar output
        #[arg(long)]
        disable_progress: bool,

        /// UDP broadcast port used for discovery
        #[arg(short, long, default_value_t = BROADCAST_PORT)]
        broadcast_port: u16,

        /// Files to send
        #[arg()]
        args: Vec<PathBuf>,
    },
}

