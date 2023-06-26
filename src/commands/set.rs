use std::str;

use super::{Command, Data, Response};
use crate::array::Value;
use crate::bulk_string::BulkString;

pub(crate) struct Set;

#[derive(PartialEq)]
enum SetOption {
    NotSpecified,
    IfExists,
    IfNotExists,
}

enum GetOption {
    NotSpecified,
    Get,
}

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
        let mut set_option = SetOption::NotSpecified;
        let mut get_option = GetOption::NotSpecified;

        for argument in &arguments[2..] {
            let arg = match argument {
                Value::BulkString(BulkString::Filled(b)) => b,
                _ => return Response::Error("invalid argument type"),
            };

            let decoded = if let Ok(decoded) = str::from_utf8(arg) {
                decoded.to_uppercase()
            } else {
                return Response::Error("invalid argument type");
            };

            match decoded.as_str() {
                "XX" => {
                    set_option = if set_option == SetOption::NotSpecified {
                        SetOption::IfExists
                    } else {
                        return Response::Error("'XX' and 'NX' can't be used at the same time");
                    }
                }
                "NX" => {
                    set_option = if set_option == SetOption::NotSpecified {
                        SetOption::IfNotExists
                    } else {
                        return Response::Error("'XX' and 'NX' can't be used at the same time");
                    }
                }
                "GET" => get_option = GetOption::Get,
                // TODO: It would be nice to be able to use non-static strings in errors, so we could do:
                // _ => return Response::Error(format!("'{decoded}' is not a valid option")),
                _ => return Response::Error("invalid option"),
            };
        }

        match (set_option, get_option) {
            (SetOption::NotSpecified, GetOption::NotSpecified) => {
                data.insert(key.clone(), value.clone());

                Response::SimpleString("OK")
            }
            (SetOption::NotSpecified, GetOption::Get) => {
                let old_value = data.get(key).unwrap_or(&BulkString::Null).clone();

                data.insert(key.clone(), value.clone());

                Response::BulkString(old_value)
            }
            (SetOption::IfExists, GetOption::NotSpecified) => {
                if data.contains_key(key) {
                    data.insert(key.clone(), value.clone());

                    return Response::SimpleString("OK");
                }

                Response::BulkString(BulkString::Null)
            }
            (SetOption::IfExists, GetOption::Get) => {
                if data.contains_key(key) {
                    let old_value = data.get(key).unwrap().clone();

                    data.insert(key.clone(), value.clone());

                    return Response::BulkString(old_value);
                }

                Response::BulkString(BulkString::Null)
            }
            (SetOption::IfNotExists, GetOption::NotSpecified) => {
                if !data.contains_key(key) {
                    data.insert(key.clone(), value.clone());

                    return Response::SimpleString("OK");
                }

                Response::BulkString(BulkString::Null)
            }
            (SetOption::IfNotExists, GetOption::Get) => {
                if !data.contains_key(key) {
                    data.insert(key.clone(), value.clone());

                    return Response::BulkString(BulkString::Null);
                }

                let old_value = data.get(key).unwrap().clone();

                Response::BulkString(old_value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_options() {
        let mut data = Data::new();

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert!(matches!(response, Response::SimpleString("OK")));

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            BulkString::Filled(b"value2".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );

        assert!(matches!(response, Response::SimpleString("OK")));
    }

    #[test]
    fn xx_not_met() {
        let mut data = Data::new();

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value".to_vec())),
            Value::BulkString(BulkString::Filled(b"XX".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert!(matches!(response, Response::BulkString(BulkString::Null)));
    }

    #[test]
    fn xx_met() {
        let mut data = Data::from([(
            BulkString::Filled(b"key".to_vec()),
            BulkString::Filled(b"value".to_vec()),
        )]);

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"XX".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            BulkString::Filled(b"value2".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );

        assert!(matches!(response, Response::SimpleString("OK")));
    }

    #[test]
    fn nx_not_met() {
        let mut data = Data::new();

        data.insert(
            BulkString::Filled(b"key".to_vec()),
            BulkString::Filled(b"value".to_vec()),
        );

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"NX".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            BulkString::Filled(b"value".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );

        assert!(matches!(response, Response::BulkString(BulkString::Null)));
    }

    #[test]
    fn nx_met() {
        let mut data = Data::new();

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value".to_vec())),
            Value::BulkString(BulkString::Filled(b"NX".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert!(matches!(response, Response::SimpleString("OK")));

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"NX".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert!(matches!(response, Response::BulkString(BulkString::Null)));

        assert_eq!(
            BulkString::Filled(b"value".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );
    }

    #[test]
    fn get() {
        let mut data = Data::new();

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert!(matches!(response, Response::BulkString(BulkString::Null)));

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            response,
            Response::BulkString(BulkString::Filled(b"value".to_vec()))
        );
    }

    #[test]
    fn xx_get_not_met() {
        let mut data = Data::new();

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value".to_vec())),
            Value::BulkString(BulkString::Filled(b"XX".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert!(matches!(response, Response::BulkString(BulkString::Null)));

        data.insert(
            BulkString::Filled(b"key".to_vec()),
            BulkString::Filled(b"value".to_vec()),
        );

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"XX".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            response,
            Response::BulkString(BulkString::Filled(b"value".to_vec()))
        );

        assert_eq!(
            BulkString::Filled(b"value2".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );
    }

    #[test]
    fn xx_get_met() {
        let mut data = Data::from([(
            BulkString::Filled(b"key".to_vec()),
            BulkString::Filled(b"value".to_vec()),
        )]);

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"XX".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            Response::BulkString(BulkString::Filled(b"value".to_vec())),
            response
        );

        assert_eq!(
            BulkString::Filled(b"value2".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );
    }

    #[test]
    fn nx_get_not_met() {
        let mut data = Data::from([(
            BulkString::Filled(b"key".to_vec()),
            BulkString::Filled(b"value".to_vec()),
        )]);

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"NX".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            Response::BulkString(BulkString::Filled(b"value".to_vec())),
            response
        );

        assert_eq!(
            BulkString::Filled(b"value".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );
    }

    #[test]
    fn nx_get_met() {
        let mut data = Data::new();

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value".to_vec())),
            Value::BulkString(BulkString::Filled(b"NX".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(Response::BulkString(BulkString::Null), response);

        data.insert(
            BulkString::Filled(b"key".to_vec()),
            BulkString::Filled(b"value".to_vec()),
        );

        let arguments = [
            Value::BulkString(BulkString::Filled(b"key".to_vec())),
            Value::BulkString(BulkString::Filled(b"value2".to_vec())),
            Value::BulkString(BulkString::Filled(b"NX".to_vec())),
            Value::BulkString(BulkString::Filled(b"GET".to_vec())),
        ];

        let response = Set.execute(&mut data, &arguments);

        assert_eq!(
            Response::BulkString(BulkString::Filled(b"value".to_vec())),
            response
        );

        assert_eq!(
            BulkString::Filled(b"value".to_vec()),
            data.get(&BulkString::Filled(b"key".to_vec()))
                .cloned()
                .unwrap()
        );
    }
}
