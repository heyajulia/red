use super::{Command, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Ping;

impl Command for Ping {
    fn execute(&self, values: &[Value]) -> Response {
        if values.is_empty() {
            Response::SimpleString("PONG")
        } else if values.len() == 1 {
            match &values[0] {
                Value::BulkString(bs) => match bs {
                    BulkString::Null => Response::Error("unexpected null bulk string"),
                    BulkString::Empty => Response::Error("unexpected empty bulk string"),
                    BulkString::Filled(s) => Response::BulkString(s.clone()),
                },
            }
        } else {
            Response::Error("wrong number of arguments")
        }
    }
}
