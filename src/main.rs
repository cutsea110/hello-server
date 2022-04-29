use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("bind 127.0.0.1:7878");

    for stream in listener.incoming() {
        let stream = stream.expect("get stream");
        println!("Connection established!: {:?}", stream);
    }
}
