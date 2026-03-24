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

const MAX_BULK_STRING_LENGTH: isize = 512 * 1024 * 1024;

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

        if length > MAX_BULK_STRING_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "bulk string length exceeds maximum",
            ));
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
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());
    eprintln!("Client connected: {peer}");

    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    loop {
        let frame = match read_frame(&mut reader) {
            Ok(frame) => frame,
            Err(e) => {
                if e.kind() != io::ErrorKind::UnexpectedEof {
                    eprintln!("Client {peer}: read error: {e}");
                }
                break;
            }
        };

        if let Err(e) = handle_request(&mut writer, &data, &frame) {
            if e.kind() != io::ErrorKind::ConnectionAborted {
                eprintln!("Client {peer}: write error: {e}");
            }
            break;
        }
    }

    eprintln!("Client disconnected: {peer}");
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

fn serve(listener: TcpListener, data: Arc<Mutex<Data>>) {
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

fn main() {
    let data = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379").expect("failed to bind to port 6379");

    println!("Listening on port 6379");

    serve(listener, data);
}

fn write_error(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    stream.write_all(format!("-ERR {message}\r\n").as_bytes())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::time::Duration;

    struct TestClient {
        writer: TcpStream,
        reader: BufReader<TcpStream>,
    }

    impl TestClient {
        fn connect(port: u16) -> Self {
            let stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let reader = BufReader::new(stream.try_clone().unwrap());
            TestClient {
                writer: stream,
                reader,
            }
        }

        fn send(&mut self, args: &[&str]) {
            write!(self.writer, "*{}\r\n", args.len()).unwrap();
            for arg in args {
                write!(self.writer, "${}\r\n{}\r\n", arg.len(), arg).unwrap();
            }
            self.writer.flush().unwrap();
        }

        fn read_line(&mut self) -> String {
            let mut line = String::new();
            self.reader.read_line(&mut line).unwrap();
            line.trim_end().to_string()
        }

        fn read_bulk_string(&mut self) -> Option<String> {
            let header = self.read_line();
            let len: isize = header.trim_start_matches('$').parse().unwrap();
            if len < 0 {
                return None;
            }
            let mut buf = vec![0u8; len as usize + 2];
            self.reader.read_exact(&mut buf).unwrap();
            Some(String::from_utf8_lossy(&buf[..len as usize]).to_string())
        }
    }

    fn start_test_server() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let data = Arc::new(Mutex::new(HashMap::new()));

        thread::spawn(move || serve(listener, data));

        // Give the server thread a moment to start accepting
        thread::sleep(Duration::from_millis(50));

        port
    }

    #[test]
    fn ping_pong() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["PING"]);
        assert_eq!(c.read_line(), "+PONG");
    }

    #[test]
    fn ping_with_message() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["PING", "hello"]);
        assert_eq!(c.read_bulk_string(), Some("hello".to_string()));
    }

    #[test]
    fn set_and_get() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["SET", "foo", "bar"]);
        assert_eq!(c.read_line(), "+OK");

        c.send(&["GET", "foo"]);
        assert_eq!(c.read_bulk_string(), Some("bar".to_string()));
    }

    #[test]
    fn get_missing_key() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["GET", "nonexistent"]);
        assert_eq!(c.read_line(), "$-1");
    }

    #[test]
    fn del_keys() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["SET", "a", "1"]);
        c.read_line(); // +OK

        c.send(&["SET", "b", "2"]);
        c.read_line(); // +OK

        c.send(&["DEL", "a", "b", "c"]);
        assert_eq!(c.read_line(), ":2");
    }

    #[test]
    fn quit() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["QUIT"]);
        assert_eq!(c.read_line(), "+OK");
    }

    #[test]
    fn unknown_command() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["FOOBAR"]);
        let response = c.read_line();
        assert!(response.starts_with('-'), "expected error, got: {response}");
    }

    #[test]
    fn multiple_commands_on_one_connection() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["SET", "counter", "hello"]);
        assert_eq!(c.read_line(), "+OK");

        c.send(&["GET", "counter"]);
        assert_eq!(c.read_bulk_string(), Some("hello".to_string()));

        c.send(&["SET", "counter", "world"]);
        assert_eq!(c.read_line(), "+OK");

        c.send(&["GET", "counter"]);
        assert_eq!(c.read_bulk_string(), Some("world".to_string()));
    }

    #[test]
    fn case_insensitive_commands() {
        let port = start_test_server();
        let mut c = TestClient::connect(port);

        c.send(&["ping"]);
        assert_eq!(c.read_line(), "+PONG");

        c.send(&["Ping"]);
        assert_eq!(c.read_line(), "+PONG");
    }
}
