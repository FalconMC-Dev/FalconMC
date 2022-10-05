use std::cmp;
use std::io::Write;

use bytes::BufMut;

#[derive(Debug)]
pub struct Writer<'a, B: ?Sized> {
    buf: &'a mut B,
}

impl<'a, B: ?Sized> Writer<'a, B> {
    pub fn new(buf: &'a mut B) -> Self { Self { buf } }
}

impl<'a, B: BufMut + ?Sized> Write for Writer<'a, B> {
    fn write(&mut self, src: &[u8]) -> std::io::Result<usize> {
        let n = cmp::min(self.buf.remaining_mut(), src.len());

        self.buf.put_slice(&src[0..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}
