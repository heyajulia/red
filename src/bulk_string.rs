use std::str;

use bytes::{Buf, Bytes};

const MAX_BULK_STRING_LENGTH: isize = 512 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum BulkString {
    Null,
    Empty,
    Filled(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BulkStringFormatError {
    Prefix,
    Length,
    LengthTrailer,
    Data,
}

pub(crate) fn parse(reader: &mut Bytes) -> Result<BulkString, BulkStringFormatError> {
    if read_byte(reader) != Some(b'$') {
        return Err(BulkStringFormatError::Prefix);
    }

    let length = match read_length(reader) {
        Some(length) => length,
        None => return Err(BulkStringFormatError::Length),
    };

    if !(-1..=MAX_BULK_STRING_LENGTH).contains(&length) {
        return Err(BulkStringFormatError::Length);
    }

    match length {
        -1 => {
            if !read_crlf(reader) {
                return Err(BulkStringFormatError::LengthTrailer);
            }

            Ok(BulkString::Null)
        }
        0 => {
            if !read_crlf(reader) {
                return Err(BulkStringFormatError::LengthTrailer);
            }

            if !read_crlf(reader) {
                return Err(BulkStringFormatError::LengthTrailer);
            }

            Ok(BulkString::Empty)
        }
        _ => {
            if !read_crlf(reader) {
                return Err(BulkStringFormatError::LengthTrailer);
            }

            let length = length as usize;
            if length > reader.remaining() {
                return Err(BulkStringFormatError::Data);
            }

            let bytes = reader.copy_to_bytes(length).to_vec();

            if !read_crlf(reader) {
                return Err(BulkStringFormatError::Data);
            }

            Ok(BulkString::Filled(bytes))
        }
    }
}

pub(crate) fn read_byte(reader: &mut Bytes) -> Option<u8> {
    if reader.has_remaining() {
        Some(reader.get_u8())
    } else {
        None
    }
}

pub(crate) fn read_length(reader: &mut Bytes) -> Option<isize> {
    let len = reader.iter().position(|&b| b == b'\r').unwrap_or(reader.remaining());
    let length_bytes = reader.copy_to_bytes(len);

    str::from_utf8(&length_bytes)
        .ok()
        .and_then(|s| s.parse().ok())
}

pub(crate) fn read_crlf(reader: &mut Bytes) -> bool {
    read_byte(reader) == Some(b'\r') && read_byte(reader) == Some(b'\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_bulk_string() {
        let mut reader = Bytes::from_static(b"$0\r\n\r\n");

        assert_eq!(Ok(BulkString::Empty), parse(&mut reader));
    }

    #[test]
    fn parse_null_bulk_string() {
        let mut reader = Bytes::from_static(b"$-1\r\n");

        assert_eq!(Ok(BulkString::Null), parse(&mut reader));
    }

    #[test]
    fn parse_hello_bulk_string() {
        let mut reader = Bytes::from_static(b"$5\r\nhello\r\n");

        assert_eq!(
            Ok(BulkString::Filled(b"hello".to_vec())),
            parse(&mut reader)
        );
    }

    #[test]
    fn missing_prefix() {
        let mut reader = Bytes::from_static(b"hello\r\n");
        assert_eq!(Err(BulkStringFormatError::Prefix), parse(&mut reader));
    }

    #[test]
    fn truncated_data() {
        let mut reader = Bytes::from_static(b"$10\r\nhi\r\n");
        assert_eq!(Err(BulkStringFormatError::Data), parse(&mut reader));
    }

    #[test]
    fn missing_length() {
        let mut reader = Bytes::from_static(b"$\r\n");
        assert_eq!(Err(BulkStringFormatError::Length), parse(&mut reader));
    }

    #[test]
    fn garbage_input() {
        let mut reader = Bytes::from_static(b"$abc\r\n");
        assert_eq!(Err(BulkStringFormatError::Length), parse(&mut reader));
    }
}
