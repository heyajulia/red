use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::OnceLock;
use std::thread;

use crate::array::{parse, Array, Value};
use crate::bulk_string::BulkString;
use crate::command::Command;
use crate::ping::Ping;

mod array;
mod bulk_string;
mod byte_reader;
mod command;
mod ping;
mod utils;

type Commands = HashMap<&'static str, Box<dyn Command + Send + Sync>>;

static COMMANDS: OnceLock<Commands> = OnceLock::new();

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

                                    if let Some(command) =
                                        COMMANDS.get().unwrap().get(command.to_uppercase().as_str())
                                    {
                                        stream.write_all(&command.execute(&values)).unwrap();
                                    } else {
                                        write_error(&mut stream, "unknown command");
                                    }
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
    COMMANDS.get_or_init(|| {
        let mut commands: Commands = HashMap::new();

        commands.insert("PING", Box::new(Ping));

        commands
    });

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
