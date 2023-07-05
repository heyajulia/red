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
    Error(&'static str),
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
            Response::Error(e) => {
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
                _ => return Response::Error($error),
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
