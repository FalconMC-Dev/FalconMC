use std::io::Write;

/// A [`Write`] implementation that discards all bytes written to it but keeps
/// track of how many were written.
///
/// Useful for determining the size of types that implements serialization to a
/// [`Write`], for example [fastnbt](fastnbt).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct Counter {
    count: usize,
}

impl Counter {
    /// Returns a new `Counter` with an initial value of 0.
    ///
    /// # Example
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use falcon_packet_core::special::Counter;
    ///
    /// let counter = Counter::new();
    /// assert_eq!(counter.count(), 0); // initial count of 0
    /// ```
    pub fn new() -> Self { Default::default() }

    /// Returns the amount of bytes that have been written to this counter so
    /// far.
    ///
    /// # Example
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// use std::io::Write;
    /// use falcon_packet_core::special::Counter;
    ///
    /// let mut counter = Counter::new();
    /// counter.write_all(b"Hello")?; // do a first write
    /// assert_eq!(counter.count(), 5);
    ///
    /// counter.write_all(b" world!")?; // do a second write
    /// assert_eq!(counter.count(), 12);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn count(&self) -> usize { self.count }
}

impl Write for Counter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = buf.len();
        self.count += written;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}
