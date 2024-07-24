use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::str;
use std::thread;
use std::time::Duration;
use sha1::{Sha1, Digest};
use base64::prelude::*;

mod httpparser;

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
        //print!("{:#010b} ", byte);
    }
    let result = stream.write(&frame);
    //print the error message if there is one
    match result {
        Err(e) => {
            println!("Error writing to stream: {}", e);
        }
        Ok(bytes_written) => {
            println!("Wrote {} bytes", bytes_written);
        }
        _ => {
            println!("Unknown error");
        
        }
    }
    
    //println!("{}", bytes_written);

}
fn calculate_response_key(websocket_key: &str) -> String {
    let globally_unique_identifier = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let combined = format!("{}{}", websocket_key, globally_unique_identifier);

    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();

    BASE64_STANDARD.encode(result)
}
fn read_string_stream(stream: &mut TcpStream) -> Result<String, String> {
    let mut buffer = [0; 1024];
    let result = stream.read(&mut buffer);
    if result.is_err() {
        return Err("Error reading stream".to_string());
    }
    let bytes_read = result.unwrap();
    let string: &str = str::from_utf8(&buffer).unwrap();
    Ok(string.to_string())
}
fn handshake(stream: &mut TcpStream) -> Result<(), String> {
    let mut buffer = [0; 2048];
    let read_result = stream.read(&mut buffer);
    if read_result.is_err() {
        return Err("Error reading streamm".to_string());
    }
    let bytes_read = read_result.unwrap();
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

    println!("Response key: {}", response_key);

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\nSec-WebSocket-Protocol: chat\r\n", response_key);
    let response_bytes = response.as_bytes();
    let result = stream.write(response_bytes);
    if result.is_err() {
        //probably should end the connection here and remove the stream from streams, probably add later
        return Err("Error writing to stream".to_string());
    }
    let bytes_written = result.unwrap();
    println!("Wrote {} bytes", bytes_written);
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").expect("Cannot bind");
    listener.set_nonblocking(true).expect("Cannot set non-blocking");
    //let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
    //let response_key = calculate_response_key(test_key);
    //println!("{}", calculate_response_key("dGhlIHNhbXBsZSBub25jZQ=="));
    // accept connections and process them serially
    let mut streams: Vec<TcpStream> = Vec::new();
    let mut waiting_handshakes: Vec<TcpStream> = Vec::new();
    loop {
        for stream_result in listener.incoming(){
            match stream_result {
                Ok(new_stream) => {
                    waiting_handshakes.push(new_stream);
                    println!("New connection");
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        println!("Error: {}", e);
                    }
                    break;
                }
            }
        }
        for i in 0..waiting_handshakes.len() {
            let result = handshake(&mut waiting_handshakes[i]);
            if result.is_err() {
                println!("Error: {}", result.err().unwrap());
                continue;
            }
            println!("Handshake complete");
            let stream = waiting_handshakes.swap_remove(i);
            streams.push(stream);
        }
        println!("--------New Tick---------");
        for stream in &mut streams {
            

            send_string_message(stream, "Hello from server!");
            let result = read_string_stream(stream);
            if result.is_err() {
                println!("Error: {}", result.err().unwrap());
                continue;
            }
            let string = result.unwrap();
            println!("Received: {}", string);
        }
        thread::sleep(Duration::from_secs(1));
    }
}