use std::io::{Read, Write};

use bytes::{Buf, BufMut};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{ReadError, WriteError};

/// Utility function to write a type to a given buffer
/// as nbt (in byte form).
pub fn nbt_write<T, B>(nbt: &T, buffer: &mut B) -> Result<(), WriteError>
where
    T: Serialize,
    B: BufMut,
{
    let writer = Writer::new(buffer);
    Ok(fastnbt::to_writer(writer, nbt)?)
}

/// Utility function to read a type from a given buffer
/// interpreting the bytes as nbt.
pub fn nbt_read<T, B>(buffer: &mut B) -> Result<T, ReadError>
where
    T: DeserializeOwned,
    B: Buf + ?Sized,
{
    let reader = Reader::new(buffer);
    Ok(fastnbt::from_reader(reader)?)
}

/// Utility function to compute the size of a type
/// if it were serialized as nbt.
pub fn nbt_size<T>(nbt: &T) -> usize
where
    T: Serialize,
{
    let counter = Counter::new();
    fastnbt::to_writer(counter, nbt).expect("Nbt sent should be valid!");
    counter.count()
}

struct Reader<'a, B: ?Sized> {
    buf: &'a mut B,
}

impl<'a, B: ?Sized> Reader<'a, B> {
    pub fn new(buf: &'a mut B) -> Self { Self { buf } }
}

impl<'a, B: Buf + ?Sized> Read for Reader<'a, B> {
    fn read(&mut self, dst: &mut [u8]) -> std::io::Result<usize> {
        let len = std::cmp::min(self.buf.remaining(), dst.len());

        self.buf.copy_to_slice(&mut dst[0..len]);
        Ok(len)
    }
}

#[derive(Debug)]
struct Writer<'a, B: ?Sized> {
    buf: &'a mut B,
}

impl<'a, B: ?Sized> Writer<'a, B> {
    pub fn new(buf: &'a mut B) -> Self { Self { buf } }
}

impl<'a, B: BufMut + ?Sized> Write for Writer<'a, B> {
    fn write(&mut self, src: &[u8]) -> std::io::Result<usize> {
        let n = std::cmp::min(self.buf.remaining_mut(), src.len());

        self.buf.put_slice(&src[0..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Hash)]
struct Counter {
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
