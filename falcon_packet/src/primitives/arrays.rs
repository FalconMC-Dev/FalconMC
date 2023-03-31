use bytes::{Buf, Bytes};

use crate::{PacketReadSeed, PacketSize, PacketWrite, ReadError};

impl<T, const N: usize> PacketSize for [T; N]
where
    T: PacketSize + 'static,
{
    fn size(&self) -> usize { self.as_slice().size() }
}

impl<T, const N: usize> PacketWrite for [T; N]
where
    T: PacketWrite + 'static,
{
    fn write<B>(&self, buffer: &mut B) -> Result<(), crate::WriteError>
    where
        B: bytes::BufMut,
    {
        self.as_slice().write(buffer)
    }
}

/// Utility function to read an array of bytes from a given buffer.
///
/// # Safety
/// If this function errors, the buffer is to
/// be considered **corrupt**. We don't mean Undefined
/// Behavior here but rather an impossibility to
/// correctly get the next types from the buffer.
pub fn array_read<const N: usize, B>(buffer: &mut B) -> Result<[u8; N], ReadError>
where
    B: Buf + ?Sized,
{
    let bytes = PacketReadSeed::<Bytes>::read(N, buffer)?;
    Ok(bytes.as_ref().try_into().expect("Should've read N bytes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut buffer = Bytes::from_static(&[0, 1, 3, 4, 2]);
        let bytes: [u8; 3] = array_read(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 3], bytes.as_slice());
        assert!(array_read::<3, Bytes>(&mut buffer).is_err());
    }
}
