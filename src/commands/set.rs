use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Set;

impl Command for Set {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if arguments.len() != 2 {
            return Response::Error("wrong number of arguments");
        }

        let key = match &arguments[0] {
            Value::BulkString(b) => match b {
                BulkString::Filled(_) => b,
                _ => return Response::Error("invalid argument #1"),
            },
        };

        let value = match &arguments[1] {
            Value::BulkString(b) => match b {
                BulkString::Filled(_) => b,
                _ => return Response::Error("invalid argument #2"),
            },
        };

        data.insert(key.clone(), value.clone());

        Response::SimpleString("OK")
    }
}
