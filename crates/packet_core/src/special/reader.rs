use std::{cmp, io::Read};

use bytes::Buf;

#[derive(Debug)]
pub struct Reader<'a, B: ?Sized> {
    buf: &'a mut B,
}

impl<'a, B: ?Sized> Reader<'a, B> {
    pub fn new(buf: &'a mut B) -> Self {
        Self { buf }
    }
}

impl<'a, B: Buf + ?Sized> Read for Reader<'a, B> {
    fn read(&mut self, dst: &mut [u8]) -> std::io::Result<usize> {
        let len = cmp::min(self.buf.remaining(), dst.len());

        self.buf.copy_to_slice(&mut dst[0..len]);
        Ok(len)
    }
}
