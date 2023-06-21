use crate::array::Value;
use crate::bulk_string::BulkString;
use crate::command::Command;

pub(crate) struct Ping;

impl Command for Ping {
    fn execute(&self, values: &Vec<Value>) -> Vec<u8> {
        if values.len() == 1 {
            return b"+PONG\r\n".to_vec();
        } else if values.len() == 2 {
            match &values[1] {
                Value::BulkString(bs) => match bs {
                    BulkString::Null => return b"-ERR unexpected null bulk string\r\n".to_vec(),
                    BulkString::Empty => b"-ERR unexpected empty bulk string\r\n".to_vec(),
                    BulkString::Filled(argument) => {
                        let mut response = vec![b'$'];

                        response.extend(argument.len().to_string().as_bytes());
                        response.extend(b"\r\n");
                        response.extend(argument);
                        response.extend(b"\r\n");

                        return response;
                    }
                },
            }
        } else {
            return b"-ERR wrong number of arguments\r\n".to_vec();
        }
    }
}
