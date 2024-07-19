use std::net::{TcpListener, TcpStream};
use std::io::Read;

fn handle_client(stream: &mut TcpStream) {
    // ...
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let first_float: [u8; 8] = buffer[0..8].try_into().unwrap();
    println!("Received {} bytes", bytes_read);
    println!("First 8 bytes: {:?}", f64::from_be_bytes(first_float));
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("New connection");
        handle_client(&mut stream?);
    }
    Ok(())
}