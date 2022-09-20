use std::marker::PhantomData;
use std::mem::MaybeUninit;

use bytes::{Buf, BufMut};

use crate::error::{ReadError, WriteError};
use crate::{PacketRead, PacketReadSeed, PacketSize, PacketSizeSeed, PacketWrite, PacketWriteSeed};

pub struct PacketArray<T>(PhantomData<T>);

impl<T> Default for PacketArray<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<const N: usize, T: PacketRead> PacketReadSeed for PacketArray<[T; N]> {
    type Value = [T; N];

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized,
    {
        let data = {
            // The `assume_init` is safe because the type we are
            // claiming to have initialized here is a bunch of
            // `MaybeUninit`s, which do not require initialization.
            let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

            // Dropping a `MaybeUninit` does nothing. Thus using raw pointer
            // assignment instead of `ptr::write` does not cause the old
            // uninitialized value to be dropped. Also if there is a panic during
            // this loop, we have a memory leak, but there is no memory safety
            // issue.
            for (i, elem) in data.iter_mut().enumerate() {
                match T::read(buffer) {
                    Ok(value) => elem.write(value),
                    Err(error) => {
                        for elem in data.iter_mut().take(i) {
                            // If one read fails we have to manually drop all
                            // the values that have been initialized so far
                            unsafe {
                                elem.assume_init_drop();
                            }
                        }
                        return Err(error);
                    }
                };
            }

            // Everything is initialized. Cast the array to the
            // initialized type.
            unsafe { data.as_ptr().cast::<[T; N]>().read() }
        };
        Ok(data)
    }
}

impl<const N: usize, T: PacketWrite> PacketWriteSeed for PacketArray<[T; N]> {
    fn write<B>(self, value: Self::Value, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        for element in value {
            element.write(buffer)?;
        }
        Ok(())
    }
}

impl<const N: usize, T: PacketSize> PacketSizeSeed for PacketArray<[T; N]> {
    type Value = [T; N];

    fn size(&self, value: &Self::Value) -> usize {
        value.iter().map(|n| n.size()).sum()
    }
}

impl<const N: usize> PacketWrite for [u8; N] {
    #[inline]
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        (&self[..]).write(buffer)
    }
}

impl<const N: usize> PacketSize for [u8; N] {
    #[inline]
    fn size(&self) -> usize {
        (&self[..]).size()
    }
}

impl<const N: usize> PacketRead for [u8; N] {
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized,
    {
        let mut buf = [0u8; N];
        buffer.copy_to_slice(&mut buf);
        Ok(buf)
    }
}
