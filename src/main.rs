use std::collections::HashMap;
use std::io::{self, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::array::{parse, Array, Value};
use crate::bulk_string::BulkString;
use crate::commands::*;

mod array;
mod bulk_string;
mod commands;

pub(crate) type Data = HashMap<BulkString, BulkString>;

fn handle_client(mut stream: TcpStream, data: Arc<Mutex<Data>>) {
    let mut buf = [0; 1024];

    loop {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                if let Err(_) = handle_request(&mut stream, &data, &buf[..n]) {
                    return;
                }
            }
        }
    }
}

fn handle_request(
    stream: &mut TcpStream,
    data: &Mutex<Data>,
    buf: &[u8],
) -> io::Result<()> {
    let result = match parse(buf) {
        Ok(result) => result,
        Err(_) => return write_error(stream, "an error occurred"),
    };

    match result {
        Array::Null => write_error(stream, "unexpected null array"),
        Array::Empty => write_error(stream, "unexpected empty array"),
        Array::Filled(values) => handle_command(stream, data, &values),
    }
}

fn handle_command(
    stream: &mut TcpStream,
    data: &Mutex<Data>,
    values: &[Value],
) -> io::Result<()> {
    let command_bytes = match &values[0] {
        Value::BulkString(BulkString::Filled(bytes)) => bytes,
        _ => return write_error(stream, "invalid command"),
    };

    let command_str = match str::from_utf8(command_bytes) {
        Ok(s) => s,
        Err(_) => return write_error(stream, "invalid command"),
    };

    let command_upper = command_str.to_uppercase();
    let command = match get_command(command_upper.as_str()) {
        Some(cmd) => cmd,
        None => return write_error(stream, "unknown command"),
    };

    let mut data = data.lock().unwrap_or_else(|e| e.into_inner());
    let bytes: Vec<u8> = command.execute(&mut data, &values[1..]).into();
    stream.write_all(&bytes)
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

fn write_error(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    stream.write_all(format!("-ERR {message}\r\n").as_bytes())
}
