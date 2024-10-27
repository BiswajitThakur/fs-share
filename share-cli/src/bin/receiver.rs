use share_utils::get_sender_addr;

fn main() {
    if let Ok(addr) = get_sender_addr("", b"password") {
        println!("Sender addr: {:#?}", addr);
    }
}
