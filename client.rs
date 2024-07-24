use std::net::TcpStream;
use std::io::{Write, Read};
use std::time::Duration;
use std::thread;
fn main(){
    let mut socket = TcpStream::connect("127.0.0.1:3000").unwrap();
    socket.write("GET /chat HTTP/1.1\r\nHost: server.example.com\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nOrigin: http://example.com\r\nSec-WebSocket-Protocol: chat, superchat\r\nSec-WebSocket-Version: 13".as_bytes()).unwrap();
    let mut buffer = [0; 1024];
    let result = socket.read(&mut buffer);
    if result.is_err(){
        println!("Error reading stream");
    }
    let bytes_read = result.unwrap();
    let string = std::str::from_utf8(&buffer).unwrap();
    println!("Received {} bytes", bytes_read);
    println!("Received: {}", string);
    loop {
        thread::sleep(Duration::from_secs(1));
        let read_result = socket.read(&mut buffer);
        if read_result.is_err(){
            println!("Error reading stream");
            continue;
        }
        let bytes_read = read_result.unwrap();
        let string = std::str::from_utf8(&buffer).unwrap();
        println!("Received {} bytes", bytes_read);
        println!("Received: {}", string);
        
    }
}