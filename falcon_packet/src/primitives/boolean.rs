use bytes::{Buf, BufMut};

use crate::{PacketRead, ReadError, PacketWrite, WriteError, PacketSize};

impl PacketRead for bool {
    #[inline]
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized,
    {
        if !buffer.has_remaining() {
            return Err(ReadError::NoMoreBytes);
        }
        Ok(buffer.get_u8() != 0)
    }
}

impl PacketWrite for bool {
    #[inline]
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut,
    {
        if !buffer.has_remaining_mut() {
            return Err(WriteError::EndOfBuffer);
        }
        buffer.put_u8(*self as u8);
        Ok(())
    }
}

impl PacketSize for bool {
    #[inline]
    fn size(&self) -> usize { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BytesMut, Bytes};

    #[test]
    fn test_size() {
        assert_eq!(1, true.size());
        assert_eq!(1, false.size());
    }

    #[test]
    fn test_write() {
        let mut buffer = BytesMut::new().limit(2);
        true.write(&mut buffer).unwrap();
        assert_eq!(&[1], buffer.get_ref().as_ref());
        false.write(&mut buffer).unwrap();
        assert_eq!(&[1, 0], buffer.get_ref().as_ref());
        assert!(true.write(&mut buffer).is_err());
    }

    #[test]
    fn test_read() {
        let mut buffer = Bytes::new();
        assert!(bool::read(&mut buffer).is_err());
        let mut buffer = Bytes::from_static(&[0, 1, 0]);
        assert!(bool::read(&mut buffer).unwrap() == false);
        assert!(bool::read(&mut buffer).unwrap() == true);
        assert!(bool::read(&mut buffer).unwrap() == false);
        assert!(bool::read(&mut buffer).is_err());
    }
}
