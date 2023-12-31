use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Del;

impl Command for Del {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if arguments.is_empty() {
            return Response::Error("wrong number of arguments");
        }

        let mut deleted = 0;

        for argument in arguments {
            let key = bulk_string_or_error!(argument);

            if data.remove(key).is_some() {
                deleted += 1;
            }
        }

        Response::Integer(deleted)
    }
}
