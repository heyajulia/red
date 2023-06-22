use super::{Command, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Ping;

impl Command for Ping {
    fn execute(&self, arguments: &[Value]) -> Response {
        if arguments.is_empty() {
            Response::SimpleString("PONG")
        } else if arguments.len() == 1 {
            match &arguments[0] {
                Value::BulkString(b) => match b {
                    BulkString::Null => Response::Error("unexpected null bulk string"),
                    BulkString::Empty => Response::Error("unexpected empty bulk string"),
                    BulkString::Filled(bytes) => Response::BulkString(bytes.clone()),
                },
            }
        } else {
            Response::Error("wrong number of arguments")
        }
    }
}
