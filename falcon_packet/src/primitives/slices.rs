use castaway::match_type;

use crate::primitives::{iter_size, iter_write, write_bytes};
use crate::{PacketSize, PacketWrite};

impl<T> PacketWrite for [T]
where
    T: PacketWrite + 'static,
{
    fn write<B>(&self, buffer: &mut B) -> Result<(), crate::WriteError>
    where
        B: bytes::BufMut,
    {
        match_type!(self, {
            &[bool] as value => {
                #[cfg(all(test, feature = "verbose-test"))]
                eprintln!("Using [bool]");
                // SAFETY: shouldn't cause a problem since we're not going from a byte to a boolean
                let bytes: &[u8] = unsafe{ std::slice::from_raw_parts(value.as_ptr() as *const u8, value.len()) };
                write_bytes(buffer, bytes)
            },
            &[u8] as value => {
                #[cfg(all(test, feature = "verbose-test"))]
                eprintln!("Using [u8]");
                write_bytes(buffer, value)
            },
            &[i8] as value => {
                #[cfg(all(test, feature = "verbose-test"))]
                eprintln!("Using [i8]");
                // SAFETY: https://users.rust-lang.org/t/how-to-convert-i8-to-u8/16308
                let bytes: &[u8] = unsafe{ std::slice::from_raw_parts(value.as_ptr() as *const u8, value.len()) };
                write_bytes(buffer, bytes)
            },
            rest => {
                #[cfg(all(test, feature = "verbose-test"))]
                eprintln!("Using default");
                iter_write(rest.iter(), buffer)
            }
        })
    }
}

impl<T> PacketSize for [T]
where
    T: PacketSize + 'static,
{
    fn size(&self) -> usize {
        match_type!(self, {
            &[bool] as value => {
                value.len()
            },
            &[u8] as value => {
                value.len()
            },
            &[i8] as value => {
                value.len()
            },
            rest => iter_size(rest.iter()),
        })
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_write() {
        let mut buffer = BytesMut::new();
        let bytes = [0u8, 1, 2, 3];
        (&bytes[..]).write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3], buffer.as_ref());

        let bytes = [0i8, 1, 2, 3];
        (&bytes[..]).write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3, 0, 1, 2, 3], buffer.as_ref());

        let bytes = [true, false, true, false];
        (&bytes[..]).write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3, 0, 1, 2, 3, 1, 0, 1, 0], buffer.as_ref());

        let bytes = [3i32];
        (&bytes[..]).write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3, 0, 1, 2, 3, 1, 0, 1, 0, 0, 0, 0, 3], buffer.as_ref());
    }

    #[test]
    fn test_size() {
        let bytes = [0u8, 1, 2, 3];
        assert_eq!(4, bytes.size());
        let bytes = [0i8, 1, 2, 3];
        assert_eq!(4, bytes.size());
        let bytes = [true, false, true, false];
        assert_eq!(4, bytes.size());
        let bytes = [0i32, 1, 2, 3];
        assert_eq!(16, bytes.size());
    }
}
