use std::marker::PhantomData;

use bytes::{Buf, BufMut};

use crate::error::{ReadError, WriteError};
use crate::{Bytes, PacketRead, PacketReadSeed, PacketSize, PacketSizeSeed, PacketWrite, PacketWriteSeed, VarI32};

/// Helper type to read any type `T` that implements [`From<String>`] from a
/// buffer and write any type `T` that implements [`AsRef<str>`](AsRef) to a
/// buffer.
pub struct PacketString<T> {
    size: usize,
    _marker: PhantomData<T>,
}

impl<T> PacketString<T> {
    /// Creates a new `PacketString`.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: AsRef<str>> PacketWriteSeed<'a> for PacketString<T> {
    fn write<B>(self, value: &Self::Value, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        let count = value.as_ref().chars().count();
        if count > self.size {
            Err(WriteError::StringTooLong(self.size, count))
        } else {
            VarI32::from(value.as_ref().len()).write(buffer)?;
            value.as_ref().as_bytes().write(buffer)
        }
    }
}

impl<'a, T: AsRef<str>> PacketSizeSeed<'a> for PacketString<T> {
    type Value = T;

    fn size(self, value: &Self::Value) -> usize { VarI32::from(value.as_ref().len()).size() + value.as_ref().len() }
}

impl<T: From<String>> PacketReadSeed for PacketString<T> {
    type Value = T;

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: Buf + ?Sized,
    {
        let len = usize::from(VarI32::read(buffer)?);
        if len > self.size * 4 {
            return Err(ReadError::StringTooLong(self.size * 4, len));
        }
        let buf = Bytes::new(len).read(buffer)?;
        let str = String::from_utf8(buf)?;
        let count = str.chars().count();
        if count > self.size {
            Err(ReadError::StringTooLong(self.size, count))
        } else {
            Ok(str.into())
        }
    }
}
