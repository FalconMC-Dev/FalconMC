use std::marker::PhantomData;

use crate::{PacketRead, PacketReadSeed, PacketSize, PacketSizeSeed, PacketWrite, PacketWriteSeed};

pub struct PacketVec<T> {
    size: usize,
    _marker: PhantomData<T>,
}

impl<T> Default for PacketVec<T> {
    fn default() -> Self {
        Self {
            size: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> PacketVec<T> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            _marker: PhantomData,
        }
    }
}

impl<T: PacketWrite> PacketWriteSeed for PacketVec<Vec<T>> {
    fn write<B>(self, value: Self::Value, buffer: &mut B) -> Result<(), crate::error::WriteError>
    where
        B: bytes::BufMut + ?Sized,
    {
        value.into_iter().try_for_each(|elem| elem.write(buffer))
    }
}

impl<T: PacketSize> PacketSizeSeed for PacketVec<Vec<T>> {
    type Value = Vec<T>;

    fn size(&self, value: &Self::Value) -> usize {
        value.iter().map(|elem| elem.size()).sum()
    }
}

impl<T: PacketRead> PacketReadSeed for PacketVec<Vec<T>> {
    type Value = Vec<T>;

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, crate::error::ReadError>
    where
        B: bytes::Buf + ?Sized,
    {
        let mut vec = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            vec.push(T::read(buffer)?);
        }
        Ok(vec)
    }
}
