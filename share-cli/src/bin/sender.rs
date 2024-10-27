use std::net::TcpListener;

use share_utils::get_receiver_addr;

fn main() {
    if let Ok(addr) = get_receiver_addr("Eagle BT", b"password") {
        println!("Reaciever addr: {:#?}", addr);
        if TcpListener::bind(addr.sender).is_ok() {
            println!("okkk");
        } else {
            println!("errrror");
        }
    }
}

