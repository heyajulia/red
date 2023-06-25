use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Ping;

impl Command for Ping {
    fn execute(&self, _data: &mut Data, arguments: &[Value]) -> Response {
        match arguments.len() {
            0 => Response::SimpleString("PONG"),
            1 => match &arguments[0] {
                Value::BulkString(b) => match b {
                    BulkString::Filled(_) => Response::BulkString(b.clone()),
                    _ => Response::Error("invalid argument"),
                },
            },
            _ => Response::Error("wrong number of arguments"),
        }
    }
}
