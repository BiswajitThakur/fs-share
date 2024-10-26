mod utils;

use std::net::SocketAddr;

pub use utils::{get_receiver_addr, get_sender_addr};

#[derive(Debug)]
pub struct Address {
    pub sender: SocketAddr,
    pub receiver: SocketAddr,
}

pub const PORT: u16 = 34254;
