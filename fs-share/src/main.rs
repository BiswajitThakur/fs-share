use std::net::TcpListener;

mod cli;

fn main() -> std::io::Result<()> {
    //cli::run()

    let listener = TcpListener::bind("[fe80::87a9:3b1f:c96:108a]:0").unwrap();
    println!("Addr: {}", listener.local_addr().unwrap());
    Ok(())
}
