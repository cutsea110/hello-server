use std::{
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(|| {
            receiver;
        });

        Self { id, thread }
    }
}

struct Job;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Self { workers, sender }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("bind 127.0.0.1:7878");
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.expect("get stream");
        println!("Connection established!: {:?}", stream);

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).expect("read request to buffer");

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "src/echo.html")
    } else if buffer.starts_with(sleep) {
        println!("heavy page...");
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "src/echo.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "src/404.html")
    };

    let mut file = File::open(filename).expect("open html file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("read html file contents");

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).expect("write response");
    stream.flush().expect("send response");
}
