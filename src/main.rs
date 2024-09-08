use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::str;
use std::thread;
use std::time::Duration;
use sha1::{Sha1, Digest};
use base64::{prelude::*, read};

mod httpparser;

fn send_string_message(stream: &mut TcpStream, message: &str) -> Result<(), ()> {
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
    match stream.write_all(&frame) {
        Ok(()) => {
            match stream.flush() {
                Ok(()) => {
                    return Ok(())
                }
                Err(e) => {
                    println!("Error flushing stream: {}", e);
                    return Err(())
                }
            }
        }
        Err(e) => {
            println!("Error writing to stream: {}", e);
            return Err(())
        }
    }
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

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", response_key);
    println!("Response: {}", response);
    let response_bytes = response.as_bytes();
    let result = stream.write(response_bytes).unwrap();

    //println!("Wrote {} bytes", response_bytes.len());
    println!("flushing stream");
    stream.flush().unwrap();
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").expect("Cannot bind");
    listener.set_nonblocking(true).expect("Cannot set nonblocking");
    //let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
    //let response_key = calculate_response_key(test_key);
    //println!("{}", calculate_response_key("dGhlIHNhbXBsZSBub25jZQ=="));
    // accept connections and process them serially
    let mut websockets = Vec::new();
    let mut streams = Vec::new();
    'main_loop: loop {
        for stream_result in listener.incoming(){
            println!("Incoming connection");
            match stream_result {
                Ok(stream) => {
                    //stream.set_nodelay(true).unwrap();
                    streams.push(stream);
                    println!("Connection established");
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    //println!("No connections to accept");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
        let streams_len = streams.len();
        println!("{} streams found", streams_len);
        for i in 0..streams_len {
            if !handshake(&mut streams[streams_len-i-1]).is_err() {
                println!("Handshake successful");
                let stream = streams.remove(streams_len-i-1); //could look into optimisation with swap_remove
                websockets.push(stream);
            }
            println!("{} streams remaining", streams.len());
        }
        let websockets_len = websockets.len();
        println!("{} websockets found", websockets_len);
        for i in 0..websockets_len {
            let mut websocket = &mut websockets[websockets_len-i-1];
            if send_string_message(websocket, "Hello, World!").is_err() {
                println!("Error sending message to websocket client removing websocket");
                websockets.remove(websockets_len-i-1); //could look into optimisation with swap_remove
            }
            else {
                println!("Message sent");
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
    
}