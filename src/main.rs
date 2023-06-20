use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;

use array::{Array, Value};
use bulk_string::BulkString;

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
                    match result {
                        Array::Null => stream.write_all(b"-ERR unexpected null array\r\n").unwrap(),
                        Array::Empty => stream
                            .write_all(b"-ERR unexpected empty array\r\n")
                            .unwrap(),
                        Array::Filled(values) => match &values[0] {
                            Value::BulkString(bs) => match bs {
                                BulkString::Null => stream
                                    .write_all(b"-ERR unexpected null bulk string\r\n")
                                    .unwrap(),
                                BulkString::Empty => stream
                                    .write_all(b"-ERR unexpected empty bulk string\r\n")
                                    .unwrap(),
                                BulkString::Filled(command) => {
                                    let command = match str::from_utf8(command) {
                                        Ok(command) => command,
                                        Err(_) => {
                                            stream.write_all(b"-ERR invalid command\r\n").unwrap();
                                            continue;
                                        }
                                    };

                                    match command.to_uppercase().as_str() {
                                        "PING" => {
                                            // TODO: Implement diadic version of PING: https://redis.io/commands/ping/
                                            stream.write_all(b"+PONG\r\n").unwrap()
                                        }
                                        _ => stream.write_all(b"-ERR unknown command\r\n").unwrap(),
                                    };
                                }
                            },
                        },
                    };
                } else {
                    stream.write_all(b"-ERR an error occurred\r\n").unwrap();
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
