use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};

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
    pub fn run(self) {
        match self.command {
            Commands::Send => {}
            Commands::Receive => {}
        }
    }
}
