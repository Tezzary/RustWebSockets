use std::net::TcpStream;
use std::io::Write;
fn main(){
    let mut socket = TcpStream::connect("127.0.0.1:3000").unwrap();
    socket.write(&f64::to_be_bytes(3.14159265358979323846264338327950)).unwrap();
}