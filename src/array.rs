use crate::bulk_string::{parse as parse_bulk_string, read_crlf, read_length, BulkString};
use crate::byte_reader::ByteReader;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Array {
    Null,
    Empty,
    Filled(Vec<Value>),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Value {
    BulkString(BulkString),
    // TODO: Other types
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ArrayFormatError {
    Prefix,
    Length,
    Data,
    LengthTrailer,
}

fn parse_value(reader: &mut ByteReader) -> Result<Value, ArrayFormatError> {
    match reader.peek_byte() {
        Some(b'$') => match parse_bulk_string(reader) {
            Ok(bulk_string) => Ok(Value::BulkString(bulk_string)),
            Err(_) => Err(ArrayFormatError::Data),
        },
        _ => Err(ArrayFormatError::Data),
    }
}

pub(crate) fn parse(data: &[u8]) -> Result<Array, ArrayFormatError> {
    let mut reader = ByteReader::new(data);

    if reader.read_byte() != Some(b'*') {
        return Err(ArrayFormatError::Prefix);
    }

    let length = match read_length(&mut reader) {
        Some(length) => length,
        None => return Err(ArrayFormatError::Length),
    };

    match length {
        -1 => {
            if reader.bytes_remaining() != 2 {
                return Err(ArrayFormatError::Data);
            }

            if !read_crlf(&mut reader) {
                return Err(ArrayFormatError::LengthTrailer);
            }

            Ok(Array::Null)
        }
        0 => {
            if reader.bytes_remaining() != 2 {
                return Err(ArrayFormatError::Data);
            }

            if !read_crlf(&mut reader) {
                return Err(ArrayFormatError::LengthTrailer);
            }

            Ok(Array::Empty)
        }
        _ => {
            if !read_crlf(&mut reader) {
                return Err(ArrayFormatError::LengthTrailer);
            }

            let mut values = Vec::with_capacity(length as usize);

            for _ in 0..length {
                values.push(parse_value(&mut reader)?);
            }

            if reader.bytes_remaining() != 0 {
                return Err(ArrayFormatError::Data);
            }

            Ok(Array::Filled(values))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_array() {
        assert_eq!(Ok(Array::Empty), parse(b"*0\r\n"));
    }

    #[test]
    fn parse_two_bulk_strings_array() {
        assert_eq!(
            Ok(Array::Filled(vec![
                Value::BulkString(BulkString::Filled(b"hello".to_vec())),
                Value::BulkString(BulkString::Filled(b"world".to_vec())),
            ])),
            parse(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n")
        );
    }

    #[test]
    #[ignore]
    fn parse_three_integers_array() {
        todo!()
    }

    #[test]
    #[ignore]
    fn parse_four_integers_and_a_bulk_string_array() {
        todo!()
    }

    #[test]
    fn parse_null_array() {
        assert_eq!(Ok(Array::Null), parse(b"*-1\r\n"));
    }

    #[test]
    #[ignore]
    fn parse_nested_array() {
        todo!()
    }

    #[test]
    #[ignore]
    fn parse_null_elements_in_array() {
        todo!()
    }
}
