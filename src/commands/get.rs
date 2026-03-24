use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Get;

impl Command for Get {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if arguments.len() != 1 {
            return Response::Error("wrong number of arguments".into());
        }

        let key = bulk_string_or_error!(&arguments[0]);

        match data.get(key) {
            Some(value) => Response::BulkString(value.clone()),
            None => Response::BulkString(BulkString::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs(s: &str) -> BulkString {
        BulkString::Filled(s.as_bytes().to_vec())
    }

    #[test]
    fn existing_key() {
        let mut data = Data::from([(bs("key"), bs("value"))]);
        let args = &[Value::BulkString(bs("key"))];
        assert_eq!(Get.execute(&mut data, args), Response::BulkString(bs("value")));
    }

    #[test]
    fn missing_key() {
        let mut data = Data::new();
        let args = &[Value::BulkString(bs("key"))];
        assert_eq!(
            Get.execute(&mut data, args),
            Response::BulkString(BulkString::Null)
        );
    }

    #[test]
    fn wrong_arg_count() {
        let mut data = Data::new();
        assert!(matches!(Get.execute(&mut data, &[]), Response::Error(_)));
    }
}
