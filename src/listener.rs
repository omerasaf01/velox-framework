use std::io::{Error, Read};
use std::net::TcpListener;

pub fn listen() -> Result<(), Error> {
    let listener = TcpListener::bind(("127.0.0.1", 8080))?;
    let port = listener.local_addr()?;
    println!("Listening on {}", port);
    let (mut tcp_stream, addr) = listener.accept()?;
    tcp_stream.set_nodelay(true)?;
    let mut input = String::new();
    let _ = tcp_stream.read_to_string(&mut input)?;
    println!("{}", input);

    Ok(())
}