use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::str;
use std::thread;
use std::time::Duration;
use sha1::{Sha1, Digest};
use base64::{prelude::*, read};

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
    stream.write_all(&frame).unwrap();
    //print the error message if there is one
    /* 
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
    */
    stream.flush().unwrap();
    
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

    let response = format!("HTTP/1.1 101 Switching Protocols\nUpgrade: websocket\nConnection: Upgrade\nSec-WebSocket-Accept: {}\nSec-WebSocket-Protocol: chat\n", response_key);
    let response_bytes = response.as_bytes();
    let result = stream.write_all(response_bytes);
    if result.is_err() {
        //probably should end the connection here and remove the stream from streams, probably add later
        return Err("Error writing to stream".to_string());
    }
    println!("Wrote {} bytes", response_bytes.len());
    println!("flushing stream");
    (*stream).flush().unwrap();
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").expect("Cannot bind");
    //let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
    //let response_key = calculate_response_key(test_key);
    //println!("{}", calculate_response_key("dGhlIHNhbXBsZSBub25jZQ=="));
    // accept connections and process them serially
    for stream_result in listener.incoming(){
        match stream_result {
            Ok(mut stream) => {
                stream.set_nodelay(true).unwrap();
                handshake(&mut stream).unwrap();
                println!("Connection established");
                thread::sleep(Duration::from_secs(3));
                for i in 0..1000 {
                    send_string_message(&mut stream, &format!("Hello, World! {}", i));
                }
                //send_string_message(&mut stream, "Hello, World!");
                println!("Sent message");
                
                let mut buffer = [0; 1024];
                
                /*
                let read_result = stream.read(&mut buffer);
                if read_result.is_err() {
                    println!("Error reading stream");
                }
                let bytes_read = read_result.unwrap();
                //let text = 
                println!("Received {} bytes", bytes_read);
                let string = str::from_utf8(&buffer).unwrap();
                println!("Received {} bytes", bytes_read);
                */
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}