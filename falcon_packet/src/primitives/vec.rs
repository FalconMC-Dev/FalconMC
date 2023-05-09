use bytes::Buf;

use crate::{PacketRead, PacketSize, PacketWrite, ReadError};

/// Deserialize into any [`FromIterator<T>`] based on the amount of
/// items `T` to expect.
///
/// # Safety
/// If this function errors, the buffer is to
/// be considered **corrupt**. We don't mean Undefined
/// Behavior here but rather an impossibility to
/// correctly get the next types from the buffer.
///
/// # Performance
/// Please do **not** use this function for reading byte arrays
/// from the network. Use [`PacketBytes`](super::PacketBytes) or
/// [`Bytes`](bytes::Bytes) instead.
pub fn iter_read<I, T, B>(len: usize, buffer: &mut B) -> Result<I, ReadError>
where
    T: PacketRead,
    I: FromIterator<T>,
    B: Buf + ?Sized,
{
    std::iter::repeat_with(|| T::read(buffer))
        .take(len)
        .collect::<Result<I, crate::error::ReadError>>()
}

impl<T> PacketWrite for Vec<T>
where
    T: PacketWrite + 'static,
{
    fn write<B>(&self, buffer: &mut B) -> Result<(), crate::WriteError>
    where
        B: bytes::BufMut + ?Sized,
    {
        self.as_slice().write(buffer)
    }
}

impl<T> PacketSize for Vec<T>
where
    T: PacketSize + 'static,
{
    fn size(&self) -> usize { self.as_slice().size() }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn test_read() {
        let mut buffer = Bytes::from_static(&[0, 0, 0, 2, 0, 0, 0, 3]);
        let nums: Vec<u16> = iter_read(4, &mut buffer).unwrap();
        assert_eq!(&[0, 2, 0, 3], &nums[..]);

        let mut buffer = Bytes::from_static(&[0, 0, 0, 2, 0, 0]);
        assert_eq!(Err(ReadError::NoMoreBytes), iter_read::<Vec<u16>, u16, Bytes>(4, &mut buffer));
    }
}
