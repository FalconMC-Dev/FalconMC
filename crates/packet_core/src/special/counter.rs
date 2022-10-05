use std::io::Write;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct Counter {
    count: usize,
}

impl Counter {
    pub fn new() -> Self { Default::default() }

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
