use crate::byte_reader::ByteReader;
use crate::utils::{read_crlf, read_length};

const MAX_BULK_STRING_LENGTH: isize = 512 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
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

pub(crate) fn parse(reader: &mut ByteReader) -> Result<BulkString, BulkStringFormatError> {
    if reader.read_byte() != Some(b'$') {
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

            let mut bytes = vec![0; length as usize];
            bytes.copy_from_slice(reader.slice(length as usize));

            if !read_crlf(reader) {
                return Err(BulkStringFormatError::Data);
            }

            Ok(BulkString::Filled(bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_bulk_string() {
        let mut reader = ByteReader::new(b"$0\r\n\r\n");

        assert_eq!(Ok(BulkString::Empty), parse(&mut reader));
    }

    #[test]
    fn parse_null_bulk_string() {
        let mut reader = ByteReader::new(b"$-1\r\n");

        assert_eq!(Ok(BulkString::Null), parse(&mut reader));
    }

    #[test]
    fn parse_hello_bulk_string() {
        let mut reader = ByteReader::new(b"$5\r\nhello\r\n");

        assert_eq!(
            Ok(BulkString::Filled(b"hello".to_vec())),
            parse(&mut reader)
        );
    }
}
