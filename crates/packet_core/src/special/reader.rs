use std::cmp;
use std::io::Read;

use bytes::Buf;

/// A [`Read`] implementation that wraps around a mutable reference to a
/// [`Buf`](bytes::Buf).
///
/// Useful for reading to a type that implements deserialization from a
/// [`Read`], e.g. [fastnbt](fastnbt).
#[derive(Debug)]
pub struct Reader<'a, B: ?Sized> {
    buf: &'a mut B,
}

impl<'a, B: ?Sized> Reader<'a, B> {
    /// Creates a new `Reader`.
    ///
    /// # Example
    /// ```
    /// use std::io::Read;
    /// use bytes::Bytes;
    /// use falcon_packet_core::special::Reader;
    ///
    /// let mut buffer = Bytes::from_static(b"Hello world!"); // Any given `Buf`
    ///
    /// let mut reader = Reader::new(&mut buffer);
    /// let mut result = Vec::new();
    /// reader.read_to_end(&mut result)?;
    ///
    /// assert_eq!(result, b"Hello world!");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn new(buf: &'a mut B) -> Self { Self { buf } }
}

impl<'a, B: Buf + ?Sized> Read for Reader<'a, B> {
    fn read(&mut self, dst: &mut [u8]) -> std::io::Result<usize> {
        let len = cmp::min(self.buf.remaining(), dst.len());

        self.buf.copy_to_slice(&mut dst[0..len]);
        Ok(len)
    }
}
