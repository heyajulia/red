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

fn parse_options(options: &[Value]) -> Result<(SetOption, GetOption), Response> {
    let mut set_option = SetOption::NotSpecified;
    let mut get_option = GetOption::NotSpecified;

    for option in options {
        let opt = match option {
            Value::BulkString(BulkString::Filled(b)) => b,
            _ => return Err(Response::Error("invalid argument type")),
        };

        let decoded = str::from_utf8(opt)
            .map_err(|_| Response::Error("invalid argument type"))?
            .to_uppercase();

        match decoded.as_str() {
            "XX" => {
                set_option = if set_option == SetOption::NotSpecified {
                    SetOption::IfExists
                } else {
                    return Err(Response::Error(
                        "'XX' and 'NX' can't be used at the same time",
                    ));
                }
            }
            "NX" => {
                set_option = if set_option == SetOption::NotSpecified {
                    SetOption::IfNotExists
                } else {
                    return Err(Response::Error(
                        "'XX' and 'NX' can't be used at the same time",
                    ));
                }
            }
            "GET" => get_option = GetOption::Get,
            // TODO: It would be nice to be able to use non-static strings in errors, so we could do:
            // Err(Response::Error(format!("'{decoded}' is not a valid option"))),
            _ => return Err(Response::Error("invalid option")),
        };
    }

    Ok((set_option, get_option))
}

impl Command for Set {
    fn execute(&self, data: &mut Data, arguments: &[Value]) -> Response {
        if !(2..=4).contains(&arguments.len()) {
            return Response::Error("wrong number of arguments");
        }

        let key = bulk_string_or_error!(&arguments[0], "invalid argument #1");
        let value = bulk_string_or_error!(&arguments[1], "invalid argument #2");

        let set_option;
        let get_option;

        match parse_options(&arguments[2..]) {
            Ok((s, g)) => {
                set_option = s;
                get_option = g;
            }
            Err(e) => return e,
        };

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

    macro_rules! bulk_string {
        (null) => {
            BulkString::Null
        };
        ($value:expr) => {
            BulkString::Filled(<[u8]>::to_vec($value.as_bytes()))
        };
    }

    macro_rules! arguments {
        ($($value:expr),*) => {
            &[$(Value::BulkString(bulk_string!($value))),*]
        };
    }

    #[test]
    fn no_options() {
        let mut data = Data::new();

        let response = Set.execute(&mut data, arguments!["key", "value"]);

        assert!(matches!(response, Response::SimpleString("OK")));

        let response = Set.execute(&mut data, arguments!["key", "value2"]);

        assert_eq!(
            bulk_string!("value2"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );

        assert!(matches!(response, Response::SimpleString("OK")));
    }

    #[test]
    fn xx_not_met() {
        let mut data = Data::new();

        let response = Set.execute(&mut data, arguments!["key", "value", "XX"]);

        assert!(data.is_empty());

        assert!(matches!(response, Response::BulkString(bulk_string!(null))));
    }

    #[test]
    fn xx_met() {
        let mut data = Data::from([(bulk_string!("key"), bulk_string!("value"))]);

        let response = Set.execute(&mut data, arguments!["key", "value2", "XX"]);

        assert_eq!(
            bulk_string!("value2"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );

        assert!(matches!(response, Response::SimpleString("OK")));
    }

    #[test]
    fn nx_not_met() {
        let mut data = Data::from([(bulk_string!("key"), bulk_string!("value"))]);

        let response = Set.execute(&mut data, arguments!["key", "value2", "NX"]);

        assert_eq!(
            bulk_string!("value"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );

        assert!(matches!(response, Response::BulkString(bulk_string!(null))));
    }

    #[test]
    fn nx_met() {
        let mut data = Data::new();

        let response = Set.execute(&mut data, arguments!["key", "value", "NX"]);

        assert!(matches!(response, Response::SimpleString("OK")));

        let response = Set.execute(&mut data, arguments!["key", "value2", "NX"]);

        assert!(matches!(response, Response::BulkString(bulk_string!(null))));

        assert_eq!(
            bulk_string!("value"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );
    }

    #[test]
    fn get() {
        let mut data = Data::new();

        let response = Set.execute(&mut data, arguments!["key", "value", "GET"]);

        assert!(matches!(response, Response::BulkString(bulk_string!(null))));

        let response = Set.execute(&mut data, arguments!["key", "value2", "GET"]);

        assert_eq!(response, Response::BulkString(bulk_string!("value")));
    }

    #[test]
    fn xx_get_not_met() {
        let mut data = Data::new();

        let response = Set.execute(&mut data, arguments!["key", "value", "XX", "GET"]);

        assert!(matches!(response, Response::BulkString(bulk_string!(null))));

        data.insert(bulk_string!("key"), bulk_string!("value"));

        let response = Set.execute(&mut data, arguments!["key", "value2", "XX", "GET"]);

        assert_eq!(response, Response::BulkString(bulk_string!("value")));

        assert_eq!(
            bulk_string!("value2"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );
    }

    #[test]
    fn xx_get_met() {
        let mut data = Data::from([(bulk_string!("key"), bulk_string!("value"))]);

        let response = Set.execute(&mut data, arguments!["key", "value2", "XX", "GET"]);

        assert_eq!(Response::BulkString(bulk_string!("value")), response);

        assert_eq!(
            bulk_string!("value2"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );
    }

    #[test]
    fn nx_get_not_met() {
        let mut data = Data::from([(bulk_string!("key"), bulk_string!("value"))]);

        let response = Set.execute(&mut data, arguments!["key", "value2", "NX", "GET"]);

        assert_eq!(Response::BulkString(bulk_string!("value")), response);

        assert_eq!(
            bulk_string!("value"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );
    }

    #[test]
    fn nx_get_met() {
        let mut data = Data::new();

        let response = Set.execute(&mut data, arguments!["key", "value", "NX", "GET"]);

        assert_eq!(Response::BulkString(bulk_string!(null)), response);

        let response = Set.execute(&mut data, arguments!["key", "value2", "NX", "GET"]);

        assert_eq!(Response::BulkString(bulk_string!("value")), response);

        assert_eq!(
            bulk_string!("value"),
            data.get(&bulk_string!("key")).cloned().unwrap()
        );
    }
}
