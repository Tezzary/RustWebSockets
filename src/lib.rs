use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;

mod handshake;

pub enum Frametype {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
    Reserved,
    Unknown
}

pub fn get_frame_type(byte: u8) -> Frametype {
    match byte & 0b00001111 {
        0 => Frametype::Continuation,
        1 => Frametype::Text,
        2 => Frametype::Binary,
        8 => Frametype::Close,
        9 => Frametype::Ping,
        10 => Frametype::Pong,
        _ => Frametype::Reserved
    }
}

pub struct Frame {
    pub fin: bool,
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub frame_type: Frametype,
    pub message_length: u64,
    pub mask: [u8; 4],
    pub data: Vec<u8>
}

fn create_frame(fin: bool, rsv1: bool, rsv2: bool, rsv3: bool, frame_type: Frametype, message_length: u64, mask: [u8; 4], data: Vec<u8>) -> Frame {
    Frame {
        fin,
        rsv1,
        rsv2,
        rsv3,
        frame_type,
        message_length,
        mask,
        data
    }
}
fn create_frames_from_buffer(buffer: &[u8]) -> Frame {
    let fin = buffer[0] & 0b10000000 != 0;
    let rsv1 = buffer[0] & 0b01000000 != 0;
    let rsv2 = buffer[0] & 0b00100000 != 0;
    let rsv3 = buffer[0] & 0b00010000 != 0;
    let frame_type = get_frame_type(buffer[0]);
    let mut message_length: u64 = 0;
    let mut i = 1;
    if buffer[i] & 0b01111111 <= 125 {
        message_length = (buffer[i] & 0b01111111) as u64;
        i += 1;
    }
    else if buffer[i] & 0b01111111 == 126 {
        let start = i+1;
        i = i+3;
        let cropped_buffer: &[u8; 2] = &buffer[start..i].try_into().unwrap();
        message_length = u16::from_be_bytes(*cropped_buffer) as u64
    }
    else if buffer[i] & 0b01111111 == 127 {
        let start = i+1;
        i = i+9;
        let cropped_buffer: &[u8; 8] = &buffer[start..i].try_into().unwrap();
        message_length = u64::from_be_bytes(*cropped_buffer);
    }
    let mask = &buffer[i..i+4];
    i += 4;
    let unmasked_data = &buffer[i..i+message_length as usize];
    let mut temp_i = 0;
    let mut j;
    let mut data: Vec<u8> = Vec::new();
    while temp_i < message_length {
        j = (temp_i % 4) as usize;
        data.push(unmasked_data[temp_i as usize] ^ mask[j]);
        temp_i += 1;
    }
    Frame {
        fin,
        rsv1,
        rsv2,
        rsv3,
        frame_type,
        message_length,
        mask: mask.try_into().unwrap(),
        data
    }
}
fn send_frame(stream: &mut TcpStream, frame: Frame) -> Result<(), ()> {
    let flags = 0b10000000 | (frame.frame_type as u8);
    let mut build_frame: Vec<u8> = Vec::new();
    build_frame.push(flags);
    if frame.message_length <= 125 {
        build_frame.push(frame.message_length as u8);
    }
    else if frame.message_length <= 65536 - 1 {//2^16 - 1 
        build_frame.push(126);
        let byte_array = u16::to_be_bytes(frame.message_length as u16); //i think big endian is right but may need confirmation
        build_frame.extend_from_slice(&byte_array);
    }
    else if frame.message_length <= 18_446_744_073_709_551_61 - 1 { //2^64 - 1, this is more than 17 million TB, honestly could just make this an else statement lmao, a message should never be this large
        build_frame.push(127);
        let byte_array = u64::to_be_bytes(frame.message_length as u64); //i think big endian is right but may need confirmation
        build_frame.extend_from_slice(&byte_array);
    }
    //masking-key left blank at 4 bytes
    build_frame.extend_from_slice(&frame.data);
    match stream.write_all(&build_frame) {
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

fn accept_new_streams(listener: &TcpListener, streams: &mut Vec<TcpStream>) {
    for stream_result in listener.incoming(){
        match stream_result {
            Ok(stream) => {
                //stream.set_nodelay(true).unwrap();
                stream.set_nonblocking(true).unwrap();
                streams.push(stream);
                println!("Connection established");
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}

pub struct Manager {
    pub listener: TcpListener,
    //pub websockets: Vec<TcpStream>,
    pub streams: Vec<TcpStream>
}

impl Manager {
    pub fn update(&mut self) -> Vec<WebSocket> {
        accept_new_streams(&self.listener, &mut self.streams);
        let mut websockets = Vec::new();
        for stream in handshake::handshake_streams(&mut self.streams) {
            websockets.push(WebSocket{stream});
        }
        websockets
    }
}
pub struct WebSocket {
    stream: TcpStream
}
impl WebSocket {
    pub fn get_messages(&mut self) -> Vec<Frame> {
        let mut buffer = [0; 16384];
        let result = self.stream.read(&mut buffer);
        if result.is_err() {
            return Vec::new();
        }
        let bytes_read = result.unwrap();
    
        
        let mut message_count = 0;
    
        let mut i = 0;
    
        let mut frames: Vec<Frame> = Vec::new();
        while i < bytes_read {
            if buffer[i] & 0b11110000 != 0b10000000 {
                println!("fin, rsv1, rsv2, rsv3 not recognised values");
                return frames;
            }
            let frame_type = get_frame_type(buffer[i]);
            
            message_count += 1;
    
            //not accounting for mask first bit of message length
    
            i += 1; 
            let mut message_length: u64 = 0;
            if buffer[i] & 0b01111111 <= 125 {
                message_length = (buffer[i] & 0b01111111) as u64;
                i += 1;
            }
            else if buffer[i] == 126 {
                let start = i+1;
                i = i+3;
                let cropped_buffer: &[u8; 2] = &buffer[start..i].try_into().unwrap();
                message_length = u16::from_be_bytes(*cropped_buffer) as u64
            }
            else if buffer[i] == 127 {
                let start = i+1;
                i = i+9;
                let cropped_buffer: &[u8; 8] = &buffer[start..i].try_into().unwrap();
                message_length = u64::from_be_bytes(*cropped_buffer);
            }
            //maybe dodgey edge cases
    
            let mask = &buffer[i..i+4];
    
            i += 4;
    
            let unmasked_data = &buffer[i..i+message_length as usize];
    
            let mut temp_i = 0;
            let mut j;
            
            let mut data: Vec<u8> = Vec::new();
            while temp_i < message_length {
                j = (temp_i % 4) as usize;
                data.push(unmasked_data[temp_i as usize] ^ mask[j]);
                temp_i += 1;
            }
    
            i += message_length as usize;
     
            let arr_data = data.as_slice();
    
            frames.push(create_frame(true, false, false, false, frame_type, message_length, mask.try_into().unwrap(), data));
        }
        frames
    }
    pub fn send_string_message(&mut self, message: &str) -> Result<(), ()> {
        let message_bytes = message.as_bytes();
        let frame = create_frame(true, false, false, false, Frametype::Text, message_bytes.len() as u64, [0, 0, 0, 0], message_bytes.to_vec());
        send_frame(&mut self.stream, frame)
    }
    pub fn send_binary_message(&mut self, message: &[u8]) -> Result<(), ()> {
        let frame = create_frame(true, false, false, false, Frametype::Binary, message.len() as u64, [0, 0, 0, 0], message.to_vec());
        send_frame(&mut self.stream, frame)
    }
}
pub fn init() -> Manager {
    let listener = TcpListener::bind("127.0.0.1:3000").expect("Cannot bind");
    listener.set_nonblocking(true).expect("Cannot set nonblocking");
    //let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
    //let response_key = calculate_response_key(test_key);
    //println!("{}", calculate_response_key("dGhlIHNhbXBsZSBub25jZQ=="));
    // accept connections and process them serially
    //let mut websockets: Vec<TcpStream> = Vec::new();
    let mut streams: Vec<TcpStream> = Vec::new();

    Manager {
        listener,
        //websockets,
        streams
    }
}