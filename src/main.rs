use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::array::{parse, Array, Value};
use crate::bulk_string::BulkString;
use crate::commands::*;

mod array;
mod bulk_string;
mod byte_reader;
mod commands;

pub(crate) type Data = HashMap<BulkString, BulkString>;

fn handle_client(mut stream: TcpStream, data: Arc<Mutex<Data>>) {
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

                                    if let Some(command) =
                                        get_command(command.to_uppercase().as_str())
                                    {
                                        let mut data = data.lock().expect("failed to acquire lock");

                                        let bytes: Vec<u8> =
                                            command.execute(&mut data, &values[1..]).into();

                                        stream.write_all(&bytes).unwrap();
                                    } else {
                                        write_error(&mut stream, "unknown command");
                                    }
                                }
                            },
                        },
                    };
                } else {
                    write_error(&mut stream, "an error occurred");
                }
            }
            Err(_) => return,
        }
    }
}

fn main() {
    let data = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379").expect("failed to bind to port 6379");

    println!("Listening on port 6379");

    for stream in listener.incoming() {
        let data = Arc::clone(&data);

        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream, data);
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
