use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) use crate::Data;

pub(crate) trait Command {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response;
}

pub(crate) enum Response {
    SimpleString(&'static str),
    Error(&'static str),
    BulkString(BulkString),
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
        }
    }
}

pub(crate) mod get;
pub(crate) mod ping;
pub(crate) mod set;

pub(crate) use get::Get;
pub(crate) use ping::Ping;
pub(crate) use set::Set;
