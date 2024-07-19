use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::str;

mod httpparser;

fn handle_socket(stream: &mut TcpStream) {
    // ...
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let first_float: [u8; 8] = buffer[0..8].try_into().unwrap();
    println!("Received {} bytes", bytes_read);
    println!("First 8 bytes: {:?}", f64::from_be_bytes(first_float));
}

fn handshake(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let string = str::from_utf8(&buffer).unwrap();
    let websocket_key = httpparser::get_header_value(string, "Sec-WebSocket-Key").unwrap();
    println!("Received {} bytes", bytes_read);
    println!("Received: {}", string);
    println!("WebSocket key: {}", websocket_key);
    let response
    stream.write(response).unwrap();
}
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;

    // accept connections and process them serially
    for incoming_stream in listener.incoming() {
        println!("New connection");
        let mut stream = incoming_stream.unwrap();
        handshake(&mut stream);
        handle_socket(&mut stream);
    }
    Ok(())
}