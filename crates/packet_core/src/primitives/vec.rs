use std::iter::{self, FromIterator};
use std::marker::PhantomData;

use super::iter::PacketIter;
use crate::{PacketRead, PacketReadSeed, PacketSizeSeed, PacketWrite, PacketWriteSeed};

/// Helper type to read any type `I` that implements [`FromIterator`] over `T`
/// from a buffer and write any type `I` that implements [`IntoIterator`] over
/// `T` to a buffer.
///
/// ## Note
/// Do not use this type to write byte slices, use [`AsRefU8`](super::AsRefU8)
/// for that.
///
/// Do not use this type to read a byte vec, use [`Bytes`](super::Bytes) for
/// that.
pub struct PacketVec<T, I> {
    size: usize,
    _marker: PhantomData<T>,
    __marker: PhantomData<I>,
}

impl<T, I> Default for PacketVec<T, I> {
    fn default() -> Self { Self::new(0) }
}

impl<T, I> PacketVec<T, I> {
    /// Creates a new `PacketVec`.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            _marker: PhantomData,
            __marker: PhantomData,
        }
    }
}

impl<'a, T, I, It> PacketWriteSeed<'a> for PacketVec<T, I>
where
    T: PacketWrite + 'a,
    It: Iterator<Item = &'a T>,
    &'a I: IntoIterator<Item = &'a T, IntoIter = It> + 'a,
{
    fn write<B>(self, value: &'a Self::Value, buffer: &'a mut B) -> Result<(), crate::error::WriteError>
    where
        B: bytes::BufMut + ?Sized,
    {
        PacketIter::new(value.into_iter()).write_ref(buffer)
    }
}

impl<'a, T, I, It> PacketSizeSeed<'a> for PacketVec<T, I>
where
    T: PacketWrite + 'a,
    It: Iterator<Item = &'a T>,
    &'a I: IntoIterator<Item = &'a T, IntoIter = It> + 'a,
{
    type Value = I;

    fn size(self, value: &'a Self::Value) -> usize { PacketIter::new(value.into_iter()).size_ref() }
}

impl<T, I> PacketReadSeed for PacketVec<T, I>
where
    T: PacketRead,
    I: FromIterator<T>,
{
    type Value = I;

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, crate::error::ReadError>
    where
        B: bytes::Buf + ?Sized,
    {
        iter::repeat_with(|| T::read(buffer))
            .take(self.size)
            .collect::<Result<I, crate::error::ReadError>>()
    }
}
