use std::borrow::Cow;
use std::marker::PhantomData;
use std::ops::Deref;

use bytes::BufMut;

use crate::error::{ReadError, WriteError};
use crate::{PacketReadSeed, PacketSize, PacketSizeSeed, PacketWrite, PacketWriteSeed};

#[derive(Default)]
pub struct AsRefU8<T>(PhantomData<T>);

impl<'a> PacketWrite for &'a [u8] {
    #[inline]
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        buffer.put_slice(self);
        Ok(())
    }
}

impl<'a> PacketSize for &'a [u8] {
    #[inline]
    fn size(&self) -> usize {
        self.len()
    }
}

impl<T: AsRef<[u8]>> PacketWriteSeed for AsRefU8<T> {
    #[inline]
    fn write<B>(self, value: Self::Value, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        value.as_ref().write(buffer)
    }
}

impl<T: AsRef<[u8]>> PacketSizeSeed for AsRefU8<T> {
    type Value = T;

    #[inline]
    fn size(&self, value: &Self::Value) -> usize {
        value.as_ref().len()
    }
}

pub struct Bytes<T> {
    size: usize,
    _marker: PhantomData<T>,
}

impl<T> Bytes<T> {
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
        let mut buf = Vec::with_capacity(self.size);
        buffer.copy_to_slice(&mut buf);
        Ok(buf.into())
    }
}

impl PacketWrite for Vec<u8> {
    #[inline]
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.deref().write(buffer)
    }
}

impl PacketSize for Vec<u8> {
    #[inline]
    fn size(&self) -> usize {
        self.deref().len()
    }
}

impl<'a> PacketWrite for Cow<'a, [u8]> {
    #[inline]
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.deref().write(buffer)
    }
}

impl<'a> PacketSize for Cow<'a, [u8]> {
    #[inline]
    fn size(&self) -> usize {
        self.deref().len()
    }
}
