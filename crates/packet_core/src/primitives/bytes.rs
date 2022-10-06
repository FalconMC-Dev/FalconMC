use std::borrow::Cow;
use std::marker::PhantomData;
use std::ops::Deref;

use bytes::BufMut;

use crate::error::{ReadError, WriteError};
use crate::{PacketReadSeed, PacketSize, PacketSizeSeed, PacketWrite, PacketWriteSeed};

/// Helper type to write any type `T` that implements [`AsRef<[u8]>`](AsRef) to
/// a minecraft connection.
#[derive(Default)]
pub struct AsRefU8<T>(PhantomData<T>);

impl PacketWrite for [u8] {
    #[inline]
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        if buffer.remaining_mut() < self.len() {
            return Err(WriteError::EndOfBuffer);
        }
        buffer.put_slice(self);
        Ok(())
    }
}

impl PacketSize for [u8] {
    #[inline]
    fn size(&self) -> usize { self.len() }
}

impl<'a, T: AsRef<[u8]>> PacketWriteSeed<'a> for AsRefU8<T> {
    #[inline]
    fn write<B>(self, value: &Self::Value, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        value.as_ref().write(buffer)
    }
}

impl<'a, T: AsRef<[u8]>> PacketSizeSeed<'a> for AsRefU8<T> {
    type Value = T;

    #[inline]
    fn size(self, value: &Self::Value) -> usize { value.as_ref().len() }
}

/// Helper type to read any type `T` that implements [`From<Vec<u8>>`] from a
/// minecraft connection.
pub struct Bytes<T> {
    size: usize,
    _marker: PhantomData<T>,
}

impl<T> Bytes<T> {
    /// Creates a new `Bytes`.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            _marker: PhantomData,
        }
    }
}

impl<T: From<Vec<u8>>> PacketReadSeed for Bytes<T> {
    type Value = T;

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: bytes::Buf + ?Sized,
    {
        if buffer.remaining() < self.size {
            return Err(ReadError::NoMoreBytes);
        }
        let mut buf = Vec::with_capacity(self.size);
        buf.extend_from_slice(buffer.copy_to_bytes(self.size).as_ref());
        Ok(buf.into())
    }
}

impl PacketWrite for Vec<u8> {
    #[inline]
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.deref().write(buffer)
    }
}

impl PacketSize for Vec<u8> {
    #[inline]
    fn size(&self) -> usize { self.deref().len() }
}

impl<'a> PacketWrite for Cow<'a, [u8]> {
    #[inline]
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.deref().write(buffer)
    }
}

impl<'a> PacketSize for Cow<'a, [u8]> {
    #[inline]
    fn size(&self) -> usize { self.deref().len() }
}
