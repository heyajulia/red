use super::{Command, CommandEntry, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

struct Ping;

inventory::submit! {
    CommandEntry { name: "PING", command: &Ping }
}

impl Command for Ping {
    fn execute(&self, _data: &mut Data, arguments: &[Value]) -> Response {
        match arguments.len() {
            0 => Response::SimpleString("PONG"),
            1 => {
                let bs = bulk_string_or_error!(&arguments[0]);

                Response::BulkString(bs.clone())
            }
            _ => Response::Error("wrong number of arguments".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args() {
        let mut data = Data::new();
        assert_eq!(Ping.execute(&mut data, &[]), Response::SimpleString("PONG"));
    }

    #[test]
    fn with_message() {
        let mut data = Data::new();
        let args = &[Value::BulkString(BulkString::Filled(b"hello".to_vec()))];
        assert_eq!(
            Ping.execute(&mut data, args),
            Response::BulkString(BulkString::Filled(b"hello".to_vec()))
        );
    }

    #[test]
    fn too_many_args() {
        let mut data = Data::new();
        let args = &[
            Value::BulkString(BulkString::Filled(b"a".to_vec())),
            Value::BulkString(BulkString::Filled(b"b".to_vec())),
        ];
        assert!(matches!(Ping.execute(&mut data, args), Response::Error(_)));
    }
}
