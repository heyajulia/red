use crate::array::Value;

pub(crate) trait Command {
    fn execute(&self, values: &[Value]) -> Response;
}

pub(crate) enum Response {
    SimpleString(&'static str),
    Error(&'static str),
    BulkString(Vec<u8>),
}

// TODO: I think TryFrom would technically be more appropriate here, because the conversion can yield semantically
// invalid results (e.g., bulk strings larger than 512 MB), but what should we do in that case?

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

                vec.extend(b.len().to_string().as_bytes());
                vec.extend(b"\r\n");
                vec.extend(b);
                vec.extend(b"\r\n");

                vec
            }
        }
    }
}

pub(crate) mod ping;
pub(crate) use ping::Ping;
