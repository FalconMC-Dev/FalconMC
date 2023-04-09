use std::mem::MaybeUninit;

use bytes::{Buf, Bytes};

use crate::{PacketRead, PacketReadSeed, PacketSize, PacketWrite, ReadError};

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

/// Utility function to read an array of `T` from a given buffer.
///
/// # Performance
/// **Do not** use this function to read an array of bytes,
/// use [`bytearray_read`] for that.
///
/// # Safety
/// If this function errors, the buffer is to
/// be considered **corrupt**. We don't mean Undefined
/// Behavior here but rather an impossibility to
/// correctly get the next types from the buffer.
pub fn array_read<B, T, const N: usize>(buffer: &mut B) -> Result<[T; N], ReadError>
where
    T: PacketRead,
    B: Buf + ?Sized,
{
    let data = {
        // The `assume_init` is safe because the type we are
        // claiming to have initialized here is a bunch of
        // `MaybeUninit`s, which do not require initialization.
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Dropping a `MaybeUninit` does nothing, so if there is a panic during this
        // loop, we have a memory leak, but there is no memory safety issue.
        for (i, element) in data.iter_mut().enumerate() {
            match T::read(buffer) {
                Ok(value) => element.write(value),
                Err(error) => {
                    // If one read fails we have to manually drop all
                    // the values that have been initialized so far
                    data.iter_mut().take(i).for_each(|e| unsafe { e.assume_init_drop() });
                    return Err(error);
                },
            };
        }

        // Everything is initialized. Transmute the array to the
        // initialized type.
        unsafe { data.as_ptr().cast::<[T; N]>().read() }
    };
    Ok(data)
}

/// Utility function to read an array of bytes from a given buffer.
///
/// # Safety
/// If this function errors, the buffer is to
/// be considered **corrupt**. We don't mean Undefined
/// Behavior here but rather an impossibility to
/// correctly get the next types from the buffer.
pub fn bytearray_read<B, const N: usize>(buffer: &mut B) -> Result<[u8; N], ReadError>
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
    fn test_bytearray() {
        let mut buffer = Bytes::from_static(&[0, 1, 3, 4, 2]);
        let bytes: [u8; 3] = bytearray_read(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 3], bytes.as_slice());
        assert!(bytearray_read::<Bytes, 3>(&mut buffer).is_err());
    }

    #[test]
    fn test_array() {
        let mut buffer = Bytes::from_static(&[0, 2, 0, 4, 0, 3]);
        let bytes: [u16; 3] = array_read(&mut buffer).unwrap();
        assert_eq!(&[2, 4, 3], bytes.as_slice());
        assert_eq!(Err(ReadError::NoMoreBytes), array_read::<Bytes, u16, 3>(&mut buffer));
    }
}
