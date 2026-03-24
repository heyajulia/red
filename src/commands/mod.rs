use std::borrow::Cow;

use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) use crate::Data;

pub(crate) trait Command {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response;
}

// TODO: Change Response to a Result<... enum of variants except Error ..., String>?
#[derive(Eq, PartialEq, Debug)]
pub(crate) enum Response {
    SimpleString(&'static str),
    Error(Cow<'static, str>),
    BulkString(BulkString),
    Integer(i64),
}

// TODO: I think TryFrom would technically be more appropriate here, because the conversion can yield semantically
// invalid results (e.g., bulk strings larger than 512 MB), but what would the calling code do in that case?
impl From<Response> for Vec<u8> {
    fn from(response: Response) -> Vec<u8> {
        match response {
            Response::SimpleString(s) => {
                let mut vec = vec![b'+'];

                vec.extend(s.as_bytes());
                vec.extend(b"\r\n");

                vec
            }
            Response::Error(ref e) => {
                let mut vec = vec![b'-'];

                vec.extend(e.as_bytes());
                vec.extend(b"\r\n");

                vec
            }
            Response::BulkString(b) => {
                let mut vec = vec![b'$'];

                match b {
                    BulkString::Null => vec.extend(b"-1\r\n"),
                    BulkString::Empty => vec.extend(b"0\r\n\r\n"),
                    BulkString::Filled(bytes) => {
                        vec.extend(format!("{}\r\n", bytes.len()).as_bytes());
                        vec.extend(bytes);
                        vec.extend(b"\r\n");
                    }
                }

                vec
            }
            Response::Integer(i) => {
                let mut v = vec![b':'];

                v.extend(i.to_string().as_bytes());
                v.extend(b"\r\n");

                v
            }
        }
    }
}

pub(crate) fn get_command(command: &str) -> Option<&dyn Command> {
    match command {
        "DEL" => Some(&Del),
        "GET" => Some(&Get),
        "PING" => Some(&Ping),
        "SET" => Some(&Set),
        _ => None,
    }
}

macro_rules! bulk_string_or_error {
    ($argument:expr) => {
        bulk_string_or_error!($argument, "invalid argument")
    };
    ($argument:expr, $error:expr) => {
        match $argument {
            Value::BulkString(b) => match b {
                BulkString::Filled(_) => b,
                _ => return Response::Error($error.into()),
            },
        }
    };
}

pub(crate) mod del;
pub(crate) mod get;
pub(crate) mod ping;
pub(crate) mod set;

pub(crate) use del::Del;
pub(crate) use get::Get;
pub(crate) use ping::Ping;
pub(crate) use set::Set;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_simple_string() {
        let bytes: Vec<u8> = Response::SimpleString("OK").into();
        assert_eq!(bytes, b"+OK\r\n");
    }

    #[test]
    fn serialize_error() {
        let bytes: Vec<u8> = Response::Error("something went wrong".into()).into();
        assert_eq!(bytes, b"-something went wrong\r\n");
    }

    #[test]
    fn serialize_error_dynamic() {
        let msg = format!("unknown command '{}'", "FOO");
        let bytes: Vec<u8> = Response::Error(msg.into()).into();
        assert_eq!(bytes, b"-unknown command 'FOO'\r\n");
    }

    #[test]
    fn serialize_bulk_string_filled() {
        let bytes: Vec<u8> =
            Response::BulkString(BulkString::Filled(b"hello".to_vec())).into();
        assert_eq!(bytes, b"$5\r\nhello\r\n");
    }

    #[test]
    fn serialize_bulk_string_empty() {
        let bytes: Vec<u8> = Response::BulkString(BulkString::Empty).into();
        assert_eq!(bytes, b"$0\r\n\r\n");
    }

    #[test]
    fn serialize_bulk_string_null() {
        let bytes: Vec<u8> = Response::BulkString(BulkString::Null).into();
        assert_eq!(bytes, b"$-1\r\n");
    }

    #[test]
    fn serialize_integer() {
        let bytes: Vec<u8> = Response::Integer(42).into();
        assert_eq!(bytes, b":42\r\n");
    }

    #[test]
    fn serialize_negative_integer() {
        let bytes: Vec<u8> = Response::Integer(-1).into();
        assert_eq!(bytes, b":-1\r\n");
    }
}
