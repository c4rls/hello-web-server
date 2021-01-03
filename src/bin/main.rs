use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use hello_web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3333").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n", "hello.html")
    } else if buffer.starts_with(b"GET /sleep HTTP/1.1\r\n") {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let headers = format!("Content-Length: {}\r\n\r\n", contents.len());

    let response = format!("{}{}{}", status_line, headers, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    // println!("Response: {}", response);
    // println!("Finished");
}
