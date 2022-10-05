use bytes::BufMut;

use crate::{PacketSize, PacketWrite, WriteError};

pub struct PacketIter<I>(I);

impl<I> PacketIter<I> {
    pub fn new(iterator: I) -> Self { Self(iterator) }
}

impl<'a, T, I> PacketIter<I>
where
    T: PacketSize + 'a,
    I: Iterator<Item = &'a T>,
{
    pub fn size_ref(self) -> usize { self.0.map(|elem| elem.size()).sum() }
}

impl<T, I> PacketIter<I>
where
    T: PacketSize,
    I: Iterator<Item = T>,
{
    pub fn size_owned(self) -> usize { self.0.map(|elem| elem.size()).sum() }
}

impl<'a, T, I> PacketIter<I>
where
    T: PacketWrite + 'a,
    I: Iterator<Item = &'a T>,
{
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
    pub fn write_owned<B>(mut self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.0.try_for_each(|elem| elem.write(buffer))
    }
}
