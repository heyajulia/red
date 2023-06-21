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
                        Array::Null => write_error(&mut stream, "unexpected null array"),
                        Array::Empty => write_error(&mut stream, "unexpected empty array"),
                        Array::Filled(values) => match &values[0] {
                            Value::BulkString(bs) => match bs {
                                BulkString::Null => {
                                    write_error(&mut stream, "unexpected null bulk string")
                                }
                                BulkString::Empty => {
                                    write_error(&mut stream, "unexpected empty bulk string")
                                }
                                BulkString::Filled(command) => {
                                    let command = match str::from_utf8(command) {
                                        Ok(command) => command,
                                        Err(_) => {
                                            write_error(&mut stream, "invalid command");

                                            continue;
                                        }
                                    };

                                    match command.to_uppercase().as_str() {
                                        "PING" => {
                                            if values.len() == 1 {
                                                stream.write_all(b"+PONG\r\n").unwrap();
                                            } else if values.len() == 2 {
                                                match &values[1] {
                                                    Value::BulkString(bs) => match bs {
                                                        BulkString::Null => {
                                                            write_error(
                                                                &mut stream,
                                                                "unexpected null bulk string",
                                                            );
                                                        }
                                                        BulkString::Empty => {
                                                            write_error(
                                                                &mut stream,
                                                                "unexpected empty bulk string",
                                                            );
                                                        }
                                                        BulkString::Filled(argument) => {
                                                            let mut response = vec![b'$'];

                                                            response.extend(
                                                                argument
                                                                    .len()
                                                                    .to_string()
                                                                    .as_bytes(),
                                                            );
                                                            response.extend(b"\r\n");
                                                            response.extend(argument);
                                                            response.extend(b"\r\n");

                                                            stream.write_all(&response).unwrap();
                                                        }
                                                    },
                                                }
                                            } else {
                                                write_error(
                                                    &mut stream,
                                                    "wrong number of arguments",
                                                );
                                            }
                                        }
                                        _ => write_error(&mut stream, "unknown command"),
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

fn write_error(stream: &mut TcpStream, message: &str) {
    stream
        .write_all(format!("-ERR {message}\r\n").as_bytes())
        .unwrap();
}
