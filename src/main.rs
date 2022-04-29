use env_logger::Env;
use log::{info, trace, warn};
use std::{
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

trait FnBox {
    fn call_box(self: Box<Self>);
}
impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) {
        (*self)()
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().expect("receive");
            match message {
                Message::NewJob(job) => {
                    trace!("Worker {} got a job; executing.", id);
                    job.call_box();
                }
                Message::Terminate => {
                    trace!("Worker {} was told to terminate.", id);

                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnBox + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
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
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).expect("send job");
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        trace!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender
                .send(Message::Terminate)
                .expect("send terminate");
        }

        trace!("Shutting down all workers.");

        for worker in &mut self.workers {
            trace!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("wait worker finishing.");
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let listener = TcpListener::bind("127.0.0.1:7878").expect("bind 127.0.0.1:7878");
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.expect("get stream");
        info!("Connection established!: {:?}", stream);

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    info!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).expect("read request to buffer");

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "src/echo.html")
    } else if buffer.starts_with(sleep) {
        warn!("heavy page...");
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

    info!("GET: {}", filename);
    stream.write(response.as_bytes()).expect("write response");
    stream.flush().expect("send response");
}
