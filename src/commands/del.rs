use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Del;

impl Command for Del {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if arguments.is_empty() {
            return Response::Error("wrong number of arguments".into());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bs(s: &str) -> BulkString {
        BulkString::Filled(s.as_bytes().to_vec())
    }

    #[test]
    fn delete_existing_keys() {
        let mut data = Data::from([(bs("a"), bs("1")), (bs("b"), bs("2")), (bs("c"), bs("3"))]);
        let args = &[Value::BulkString(bs("a")), Value::BulkString(bs("c"))];
        assert_eq!(Del.execute(&mut data, args), Response::Integer(2));
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn delete_missing_key() {
        let mut data = Data::new();
        let args = &[Value::BulkString(bs("nope"))];
        assert_eq!(Del.execute(&mut data, args), Response::Integer(0));
    }

    #[test]
    fn no_args() {
        let mut data = Data::new();
        assert!(matches!(Del.execute(&mut data, &[]), Response::Error(_)));
    }
}
