use bytes::BufMut;

use crate::{PacketSize, PacketWrite, WriteError};

/// Helper type to write iterators over implementors of the seed-less flavored
/// traits to a minecraft connection.
///
/// ## Note
/// Do not use this type to write byte slices, use [`AsRefU8`](super::AsRefU8)
/// for that.
///
/// Do not use this type to read a byte vec, use [`Bytes`](super::Bytes) for
/// that.
pub struct PacketIter<I>(I);

impl<I> PacketIter<I> {
    /// Create a new `PacketIter`.
    pub fn new(iterator: I) -> Self { Self(iterator) }
}

impl<'a, T, I> PacketIter<I>
where
    T: PacketSize + 'a,
    I: Iterator<Item = &'a T>,
{
    /// Determine the size of the contained iterator. The iterator will iterate
    /// over **references** of `T`.
    pub fn size_ref(self) -> usize { self.0.map(|elem| elem.size()).sum() }
}

impl<T, I> PacketIter<I>
where
    T: PacketSize,
    I: Iterator<Item = T>,
{
    /// Determine the size of the contained iterator. The iterator will iterate
    /// over **owned values** of `T`.
    pub fn size_owned(self) -> usize { self.0.map(|elem| elem.size()).sum() }
}

impl<'a, T, I> PacketIter<I>
where
    T: PacketWrite + 'a,
    I: Iterator<Item = &'a T>,
{
    /// Write the contained iterator to a buffer. The iterator will iterate over
    /// **references** of `T`.
    pub fn write_ref<B>(mut self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.0.try_for_each(|elem| elem.write(buffer))
    }
}

impl<T, I> PacketIter<I>
where
    T: PacketWrite,
    I: Iterator<Item = T>,
{
    /// Write the contained iterator to a buffer. The iterator will iterate over
    /// **owned values** of `T`.
    pub fn write_owned<B>(mut self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.0.try_for_each(|elem| elem.write(buffer))
    }
}
