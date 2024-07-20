use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::str;
use std::thread;
use std::time::Duration;
use sha1::{Sha1, Digest};
use base64::prelude::*;

mod httpparser;

fn send_websocket_message(stream: &mut TcpStream, message: &str) {
    let message_bytes = message.as_bytes();
    let message_length = message_bytes.len();
    let mut message_frame: Vec<u8> = Vec::new();
    message_frame.push(0b10000001);
    if message_length <= 125 {
        message_frame.push(message_length as u8);
    } else if message_length <= 65535 {
        message_frame.push(126);
        message_frame.push((message_length >> 8) as u8);
        message_frame.push(message_length as u8);
    } else {
        message_frame.push(127);
        message_frame.push((message_length >> 56) as u8);
        message_frame.push((message_length >> 48) as u8);
        message_frame.push((message_length >> 40) as u8);
        message_frame.push((message_length >> 32) as u8);
        message_frame.push((message_length >> 24) as u8);
        message_frame.push((message_length >> 16) as u8);
        message_frame.push((message_length >> 8) as u8);
        message_frame.push(message_length as u8);
    }
    message_frame.extend_from_slice(message_bytes);
    stream.write(&message_frame).unwrap();
    println!("Sent: {}", message);
}

fn send_string_message(stream: &mut TcpStream, message: &str) {
    let message_bytes = message.as_bytes();
    let message_length = message_bytes.len();
    let flags = 0b10000001;
    let mut frame: Vec<u8> = Vec::new();
    frame.push(flags);
    if message_length <= 125 {
        frame.push(message_length as u8);
    }
    else if message_length <= 65536 - 1 {//2^16 - 1 
        frame.push(126);
        let byte_array = u16::to_be_bytes(message_length as u16); //i think big endian is right but may need confirmation
        /*for byte in byte_array {
            frame.push(byte);
        }*/
        frame.extend_from_slice(&byte_array);
    }
    else if message_length <= 18_446_744_073_709_551_61 - 1 { //2^64 - 1, this is more than 17 million TB, honestly could just make this an else statement lmao, a message should never be this large
        frame.push(127);
        let byte_array = u64::to_be_bytes(message_length as u64); //i think big endian is right but may need confirmation
        /*for byte in byte_array {
            frame.push(byte);
        }*/
        frame.extend_from_slice(&byte_array);
    }
    //masking-key left blank at 4 bytes
    frame.extend_from_slice(message_bytes);
    for byte in &frame {
        print!("{:#010b} ", byte);
    }
    let bytes_written = stream.write(&frame).unwrap();
    println!("{}", bytes_written);

}
fn handle_socket(stream: &mut TcpStream) {
    // ...
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let first_float: [u8; 8] = buffer[0..8].try_into().unwrap();
    println!("Received {} bytes", bytes_read);
    println!("First 8 bytes: {:?}", f64::from_be_bytes(first_float));
}
fn calculate_response_key(websocket_key: &str) -> String {
    let globally_unique_identifier = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let combined = format!("{}{}", websocket_key, globally_unique_identifier);

    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();

    BASE64_STANDARD.encode(result)
}
fn handshake(stream: &mut TcpStream) -> Result<(), String> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let string = str::from_utf8(&buffer).unwrap();
    let result_websocket_key = httpparser::get_header_value(string, "Sec-WebSocket-Key");
    if result_websocket_key.is_err() {
        return Err("No WebSocket key found".to_string());
    }
    let websocket_key = result_websocket_key.unwrap();
    //println!("Received {} bytes", bytes_read);
    //println!("Received: {}", string);
    //println!("WebSocket key: {}", websocket_key);
    
    let response_key = calculate_response_key(&websocket_key);

    //println!("Response key: {}", response_key);

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\nSec-WebSocket-Protocol: chat\r\n", response_key);
    let response_bytes = response.as_bytes();
    stream.write(response_bytes).unwrap();
    Ok(())
}

fn main() {
    let mut listener = TcpListener::bind("127.0.0.1:3000").expect("Cannot bind");
    listener.set_nonblocking(true).expect("Cannot set non-blocking");
    //let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
    //let response_key = calculate_response_key(test_key);
    //println!("{}", calculate_response_key("dGhlIHNhbXBsZSBub25jZQ=="));
    // accept connections and process them serially
    let mut streams: Vec<&mut TcpStream> = Vec::new();
    loop {
        for stream_result in listener.incoming(){
            match stream_result {
                Ok(mut new_stream) => {
                    let result = handshake(&mut new_stream);
                    if result.is_err() {
                        println!("Error: {}", result.err().unwrap());
                        continue;
                    }
                    println!("New connection");
                    streams.push(&mut new_stream);
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        println!("Error: {}", e);
                    }
                    break;
                }
            }
        }
        for stream in &mut streams {
            send_string_message(stream, "Hello from server!");
            let mut buffer: [u8; 1024] = [0; 1024];
            let bytes_read = stream.read(&mut buffer).unwrap();
            println!("Received {} bytes", bytes_read);
            //convert to string
            let mut message = String::new();
            for byte in buffer.iter() {
                message.push(*byte as char);
            }
            println!("Received: {}", message);
        }
        thread::sleep(Duration::from_secs(1));
    }
}