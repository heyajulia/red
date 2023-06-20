pub(crate) struct ByteReader<'a> {
    pub(crate) data: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> ByteReader<'a> {
        ByteReader { data, offset: 0 }
    }

    pub(crate) fn read_byte(&mut self) -> Option<u8> {
        if self.offset >= self.data.len() {
            return None;
        }

        let byte = self.data[self.offset];

        self.offset += 1;

        Some(byte)
    }

    pub(crate) fn peek_byte(&self) -> Option<u8> {
        if self.offset >= self.data.len() {
            None
        } else {
            Some(self.data[self.offset])
        }
    }

    pub(crate) fn bytes_remaining(&self) -> usize {
        self.data.len() - self.offset
    }

    pub(crate) fn read_while<F>(&mut self, mut f: F) -> &'a [u8]
    where
        F: FnMut(u8) -> bool,
    {
        let start = self.offset;

        while self.bytes_remaining() > 0 {
            if !f(self.data[self.offset]) {
                break;
            }

            self.offset += 1;
        }

        &self.data[start..self.offset]
    }

    pub(crate) fn slice(&mut self, length: usize) -> &'a [u8] {
        let s = &self.data[self.offset..self.offset + length];

        self.offset += length;

        s
    }
}
