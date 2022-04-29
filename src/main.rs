use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("bind 127.0.0.1:7878");

    for stream in listener.incoming() {
        let stream = stream.expect("get stream");
        println!("Connection established!: {:?}", stream);
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).expect("read request to buffer");
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
