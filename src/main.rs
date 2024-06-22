use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
mod constants;
use constants::*;
mod thread_pool;
use thread_pool::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    println!("Handling new connection");
    loop {
        let mut buf = vec![0; 512];
        let bytes_read = stream.read(&mut buf).unwrap();
        if bytes_read == 0 {
            println!("br");
            break;
        }
        println!("{:?}", std::str::from_utf8(&buf));
        stream.write(format!("+PONG{}", CLRF).as_bytes()).unwrap();
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let thread_pool = ThreadPool::new(3);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                thread_pool.execute(|| handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
