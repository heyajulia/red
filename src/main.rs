#![allow(dead_code)]

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::array::parse;

mod array;
mod bulk_string;
mod byte_reader;
mod utils;

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 1024];

    loop {
        match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    return;
                }

                if let Ok(result) = parse(&buf[..n]) {
                    println!("{:#?}", result);

                    stream.write_all(b"+OK\r\n").unwrap();
                } else {
                    stream.write_all(b"-ERR An error occurred\r\n").unwrap();
                }
            }
            Err(_) => return,
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").expect("failed to bind to port 6379");

    println!("Listening on port 6379");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}
