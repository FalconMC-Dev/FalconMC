use std::cmp;
use std::io::Write;

use bytes::BufMut;

/// A [`Write`] implementation that wraps around a mutable reference to
/// [`BufMut`].
///
/// Useful for writing a type that implementats serialization to [`Write`], e.g.
/// [fastnbt](fastnbt).
#[derive(Debug)]
pub struct Writer<'a, B: ?Sized> {
    buf: &'a mut B,
}

impl<'a, B: ?Sized> Writer<'a, B> {
    /// Creates a new `Writer`.
    ///
    /// # Example
    /// ```
    /// use std::io::Write;
    /// use bytes::BytesMut;
    /// use falcon_packet_core::special::Writer;
    ///
    /// let mut buffer = BytesMut::new();
    ///
    /// let mut writer = Writer::new(&mut buffer);
    /// writer.write_all(b"Hello world!")?;
    ///
    /// assert_eq!(&buffer[..], b"Hello world!");
    /// Ok::<(), std::io::Error>(())
    /// ```
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
