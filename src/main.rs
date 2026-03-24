use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
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

fn read_frame(reader: &mut BufReader<TcpStream>) -> io::Result<Vec<u8>> {
    let mut frame = Vec::new();

    // Read the array header line (*N\r\n)
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    frame.extend(line.as_bytes());

    let count: isize = line
        .trim_end_matches("\r\n")
        .strip_prefix('*')
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid array header"))?;

    if count <= 0 {
        return Ok(frame);
    }

    for _ in 0..count {
        // Read the bulk string header ($N\r\n)
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        frame.extend(line.as_bytes());

        let length: isize = line
            .trim_end_matches("\r\n")
            .strip_prefix('$')
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid bulk string header")
            })?;

        if length < 0 {
            continue;
        }

        // Read exactly length bytes + \r\n
        let total = length as usize + 2;
        let start = frame.len();
        frame.resize(start + total, 0);
        reader.read_exact(&mut frame[start..])?;
    }

    Ok(frame)
}

fn handle_client(stream: TcpStream, data: Arc<Mutex<Data>>) {
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    loop {
        let frame = match read_frame(&mut reader) {
            Ok(frame) => frame,
            Err(_) => return,
        };

        if let Err(_) = handle_request(&mut writer, &data, &frame) {
            return;
        }
    }
}

fn handle_request(stream: &mut TcpStream, data: &Mutex<Data>, buf: &[u8]) -> io::Result<()> {
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

    if command_upper == "QUIT" {
        stream.write_all(b"+OK\r\n")?;
        return Err(io::Error::from(io::ErrorKind::ConnectionAborted));
    }

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
