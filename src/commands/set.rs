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
                "XX" => set_if_exist = true,
                "NX" => set_if_not_exist = true,
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
                let old_value = data.get(key).unwrap().clone();

                data.insert(key.clone(), value.clone());

                if get {
                    return Response::BulkString(old_value);
                }

                return Response::SimpleString("OK");
            }

            return Response::BulkString(BulkString::Null);
        }

        if set_if_not_exist {
            let mut was_inserted = false;
            let mut current_value = None;

            if !data.contains_key(key) {
                was_inserted = true;

                data.insert(key.clone(), value.clone());
            } else {
                current_value = Some(data.get(key).unwrap().clone());
            }

            return if get {
                if let Some(current_value) = current_value {
                    Response::BulkString(current_value)
                } else {
                    Response::BulkString(BulkString::Null)
                }
            } else if was_inserted {
                Response::SimpleString("OK")
            } else {
                Response::BulkString(BulkString::Null)
            };
        }

        if get {
            let mut old_value = None;

            if data.contains_key(key) {
                old_value = Some(data.get(key).unwrap().clone());
            }

            data.insert(key.clone(), value.clone());

            if let Some(old_value) = old_value {
                Response::BulkString(old_value)
            } else {
                Response::BulkString(BulkString::Null)
            }
        } else {
            data.insert(key.clone(), value.clone());

            Response::SimpleString("OK")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
