use std::str;
use std::net::TcpStream;
use std::io::prelude::*;
use base64::prelude::*;
use sha1::{Sha1, Digest};

pub fn get_header_value(headers: &str, key: &str) -> Result<String, String> {
    let lines = headers.lines();
    let mut value = Err("Header not found".to_string());
    for line in lines {
        if line.starts_with(key) {
            let parts: Vec<&str> = line.split(": ").collect();
            value = Ok(parts[1].to_string());
        }
    }
    value
}
fn handshake(stream: &mut TcpStream) -> Result<(), String> {
    let mut buffer = [0; 2048];
    let read_result = stream.read(&mut buffer);
    if read_result.is_err() {
        return Err("Error reading streamm".to_string());
    }
    let bytes_read = read_result.unwrap();
    let string = str::from_utf8(&buffer).unwrap();
    let result_websocket_key = get_header_value(string, "Sec-WebSocket-Key");
    if result_websocket_key.is_err() {
        return Err("No WebSocket key found".to_string());
    }
    let websocket_key = result_websocket_key.unwrap();
    //println!("Received {} bytes", bytes_read);
    //println!("Received: {}", string);
    //println!("WebSocket key: {}", websocket_key);
    
    let response_key = calculate_response_key(&websocket_key);

    println!("Response key: {}", response_key);

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", response_key);
    println!("Response: {}", response);
    let response_bytes = response.as_bytes();
    let result = stream.write(response_bytes).unwrap();

    //println!("Wrote {} bytes", response_bytes.len());
    println!("flushing stream");
    stream.flush().unwrap();
    Ok(())
}
fn calculate_response_key(websocket_key: &str) -> String {
    let globally_unique_identifier = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let combined = format!("{}{}", websocket_key, globally_unique_identifier);

    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();

    BASE64_STANDARD.encode(result)
}
pub fn handshake_streams(streams: &mut Vec<TcpStream>) -> Vec<TcpStream> {
    let streams_len = streams.len();
    let mut new_websockets = Vec::new();
    //println!("{} streams found", streams_len);
    for i in 0..streams_len {
        if !handshake(&mut streams[streams_len-i-1]).is_err() {
            println!("Handshake successful");
            let stream = streams.remove(streams_len-i-1); //could look into optimisation with swap_remove
            new_websockets.push(stream)
        }
        println!("{} streams remaining", streams.len());
    }
    new_websockets
}