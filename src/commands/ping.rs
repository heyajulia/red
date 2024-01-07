use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Ping;

impl Command for Ping {
    fn execute(&self, _data: &mut Data, arguments: &[Value]) -> Response {
        match arguments.len() {
            0 => Response::SimpleString("PONG"),
            1 => {
                let bs = bulk_string_or_error!(&arguments[0]);

                Response::BulkString(bs.clone())
            }
            _ => Response::Error("wrong number of arguments"),
        }
    }
}
