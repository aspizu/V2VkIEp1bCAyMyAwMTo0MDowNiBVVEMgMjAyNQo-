use bytes::{Bytes, BytesMut};

#[derive(Default)]
pub struct StringPool {
    buf: BytesMut,
    strings: Vec<Bytes>,
}

impl StringPool {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_str(&mut self, string: &[u8]) {
        self.buf.extend_from_slice(string);
        self.strings.push(self.buf.split().freeze());
    }

    pub fn push(&mut self, string: Bytes) {
        self.strings.push(string);
    }

    pub fn get_strings(&self) -> &[Bytes] {
        &self.strings
    }
}
