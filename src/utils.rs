use crate::byte_reader::ByteReader;
use std::str;

pub(crate) fn read_length(reader: &mut ByteReader) -> Option<isize> {
    let length_bytes = reader.read_while(|b| b != b'\r');

    str::from_utf8(length_bytes)
        .ok()
        .and_then(|s| s.parse().ok())
}

pub(crate) fn read_crlf(reader: &mut ByteReader) -> bool {
    reader.read_byte() == Some(b'\r') && reader.read_byte() == Some(b'\n')
}
