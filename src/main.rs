use std::{
    io::{Read, Write},
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

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write(response.as_bytes()).expect("write response");
    stream.flush().expect("send response");
}
