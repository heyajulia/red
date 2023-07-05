use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Get;

impl Command for Get {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if arguments.len() != 1 {
            return Response::Error("wrong number of arguments");
        }

        let key = bulk_string_or_error!(&arguments[0]);

        match data.get(key) {
            Some(value) => Response::BulkString(value.clone()),
            None => Response::BulkString(BulkString::Null),
        }
    }
}
