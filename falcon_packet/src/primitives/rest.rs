use bytes::{Bytes, BytesMut};

use crate::PacketReadSeed;

use super::PacketBytes;

/// Reads until the end of the buffer
impl PacketReadSeed<PacketBytes> for () {
    fn read<B>(self, buffer: &mut B) -> Result<PacketBytes, crate::ReadError>
    where
        B: bytes::Buf + ?Sized
    {
        buffer.remaining().read(buffer)
    }
}

/// Reads until the end of the buffer
impl PacketReadSeed<Bytes> for () {
    fn read<B>(self, buffer: &mut B) -> Result<Bytes, crate::ReadError>
    where
        B: bytes::Buf + ?Sized
    {
        buffer.remaining().read(buffer)
    }
}

/// Reads until the end of the buffer
impl PacketReadSeed<BytesMut> for () {
    fn read<B>(self, buffer: &mut B) -> Result<BytesMut, crate::ReadError>
    where
        B: bytes::Buf + ?Sized
    {
        buffer.remaining().read(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::PacketRead;

    #[test]
    fn test_read() {
        let mut buffer = Bytes::from_static(&[0, 1, 2, 3, 4]);
        assert!(u8::read(&mut buffer).is_ok());
        let bytes: PacketBytes = ().read(&mut buffer).unwrap();
        assert_eq!(&[1, 2, 3, 4], bytes.as_ref());

        let mut buffer = Bytes::from_static(&[0, 1, 2, 3, 4]);
        assert!(u8::read(&mut buffer).is_ok());
        let bytes: Bytes = ().read(&mut buffer).unwrap();
        assert_eq!(&[1, 2, 3, 4], bytes.as_ref());

        let mut buffer = Bytes::from_static(&[0, 1, 2, 3, 4]);
        assert!(u8::read(&mut buffer).is_ok());
        let bytes: BytesMut = ().read(&mut buffer).unwrap();
        assert_eq!(&[1, 2, 3, 4], bytes.as_ref());
    }
}
