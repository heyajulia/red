use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Get;

impl Command for Get {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if arguments.len() != 1 {
            return Response::Error("wrong number of arguments");
        }

        let key = match &arguments[0] {
            Value::BulkString(b) => match b {
                BulkString::Filled(_) => b,
                _ => return Response::Error("invalid argument"),
            },
        };

        match data.get(key) {
            Some(value) => Response::BulkString(value.clone()),
            None => Response::BulkString(BulkString::Null),
        }
    }
}
