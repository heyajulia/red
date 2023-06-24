use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Set;

impl Command for Set {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if !(2..=4).contains(&arguments.len()) {
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

        // TODO: Implement more options: https://redis.io/commands/set.
        let mut set_if_exist = false;
        let mut set_if_not_exist = false;
        let mut get = false;

        for argument in &arguments[2..] {
            let arg = match argument {
                Value::BulkString(BulkString::Filled(b)) => b,
                _ => return Response::Error("invalid argument type"),
            };

            // TODO: handle this error
            let decoded = std::str::from_utf8(arg).unwrap().to_uppercase();

            match decoded.as_str() {
                "NX" => set_if_not_exist = true,
                "XX" => set_if_exist = true,
                "GET" => get = true,
                // TODO: It would be nice to be able to use non-static strings in errors, so we could do:
                // _ => return Response::Error(format!("'{decoded}' is not a valid option")),
                _ => return Response::Error("invalid option"),
            };

            if set_if_exist && set_if_not_exist {
                return Response::Error("'XX' and 'NX' can't be used at the same time");
            }
        }

        if set_if_exist {
            if data.contains_key(key) {
                data.insert(key.clone(), value.clone());

                return Response::SimpleString("OK");
            } else {
                return Response::BulkString(BulkString::Null);
            }
        }

        if set_if_not_exist {
            if !data.contains_key(key) {
                data.insert(key.clone(), value.clone());

                return Response::SimpleString("OK");
            } else {
                return Response::BulkString(BulkString::Null);
            }
        }

        data.insert(key.clone(), value.clone());

        Response::SimpleString("OK")
    }
}
