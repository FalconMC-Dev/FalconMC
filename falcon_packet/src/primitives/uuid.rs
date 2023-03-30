use std::{str::FromStr, ops::Deref};

use bytes::Bytes;
use uuid::Uuid;

use crate::{PacketRead, PacketReadSeed, PacketSize, PacketWrite};

use super::{PacketString, write_str_unchecked};

impl PacketWrite for Uuid {
    fn write<B>(&self, buffer: &mut B) -> Result<(), crate::WriteError>
    where
        B: bytes::BufMut,
    {
        super::write_bytes(buffer, self.as_ref())
    }
}

impl PacketSize for Uuid {
    #[inline]
    fn size(&self) -> usize { 16 }
}

impl PacketRead for Uuid {
    fn read<B>(buffer: &mut B) -> Result<Self, crate::ReadError>
    where
        B: bytes::Buf + ?Sized,
        Self: Sized,
    {
        let bytes = PacketReadSeed::<Bytes>::read(16, buffer)?;
        let array: &[u8; 16];
        { // This block is taken from the `TryFrom` implementation for [T; N]
            let ptr = bytes.as_ptr() as *const [u8; 16];
            // SAFETY: ok because PacketReadSeed::<Bytes> ensures there are 16 bytes read
            array = unsafe { &*ptr };
        }
        Ok(Uuid::from_bytes(*array))
    }
}

/// A specialized wrapper to represent [`Uuid`]'s as hyphenated strings
/// for the minecraft protocol.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct StringUuid {
    inner: Uuid,
}

impl StringUuid {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn into_inner(self) -> Uuid {
        self.inner
    }
}

impl Deref for StringUuid {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Uuid> for StringUuid {
    fn from(uuid: Uuid) -> Self { Self { inner: uuid} }
}

impl From<StringUuid> for Uuid {
    fn from(uuid: StringUuid) -> Self { uuid.inner }
}

/// Serialized length of a string uuid is always the same
const STR_UUID_LEN: usize = {
    // copied over from VarI32
    let n = ({ i32::BITS as usize + 6 } - (uuid::fmt::Hyphenated::LENGTH as i32).leading_zeros() as usize) / 7;
    n + uuid::fmt::Hyphenated::LENGTH
};

impl PacketRead for StringUuid {
    fn read<B>(buffer: &mut B) -> Result<Self, crate::ReadError>
    where
        B: bytes::Buf + ?Sized,
        Self: Sized
    {
        let s: PacketString = 36.read(buffer)?;
        Ok(Uuid::from_str(s.as_ref())?.into())
    }
}

impl PacketWrite for StringUuid {
    fn write<B>(&self, buffer: &mut B) -> Result<(), crate::WriteError>
    where
        B: bytes::BufMut
    {
        let mut buf = [0u8; uuid::fmt::Hyphenated::LENGTH];
        self.inner.as_hyphenated().encode_lower(&mut buf);
        // SAFETY: the [`uuid`] crate generates valid utf8!
        write_str_unchecked(buffer, unsafe { std::str::from_utf8_unchecked(&buf) })
    }
}

impl PacketSize for StringUuid {
    fn size(&self) -> usize {
        STR_UUID_LEN
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bytes::{BytesMut, BufMut, Buf};

    use crate::{ReadError, WriteError};

    use super::*;

    #[test]
    fn test_read() {
        let mut buffer = Bytes::from_static(&[0x8b, 0x9f, 0xe9, 0xc4, 0xd6, 0x4e, 0x4c, 0x2c, 0xaa, 0x21, 0x0c, 0x0c, 0xdb, 0x82, 0x92, 0x53]);
        let uuid = Uuid::read(&mut buffer).unwrap();
        assert_eq!("8b9fe9c4-d64e-4c2c-aa21-0c0cdb829253", uuid.hyphenated().to_string());
        let mut buffer = Bytes::from_static(&[0x01, 0x02]);
        assert_eq!(Err(ReadError::NoMoreBytes), Uuid::read(&mut buffer));
        let mut buffer = Bytes::from_static(&[0x8b, 0x9f, 0xe9, 0xc4, 0xd6, 0x4e, 0x4c, 0x2c, 0xaa, 0x21, 0x0c, 0x0c, 0xdb, 0x82, 0x92, 0x53, 0x24]);
        let _ = Uuid::read(&mut buffer).unwrap();
        assert_eq!(&[0x24], buffer.as_ref());
    }

    #[test]
    fn test_write() {
        let mut buffer = BytesMut::new().limit(15);
        let uuid = Uuid::from_str("8b9fe9c4-d64e-4c2c-aa21-0c0cdb829253").unwrap();
        assert_eq!(Err(WriteError::EndOfBuffer), uuid.write(&mut buffer));
        let mut buffer = BytesMut::new();
        let uuid = Uuid::from_str("8b9fe9c4-d64e-4c2c-aa21-0c0cdb829253").unwrap();
        uuid.write(&mut buffer).unwrap();
        assert_eq!(&[0x8b, 0x9f, 0xe9, 0xc4, 0xd6, 0x4e, 0x4c, 0x2c, 0xaa, 0x21, 0x0c, 0x0c, 0xdb, 0x82, 0x92, 0x53], buffer.as_ref());
    }

    #[test]
    fn test_str_read() {
        let bytes = Bytes::from_static(b"8b9fe9c4-d64e-4c2c-aa21-0c0cdb829253");
        let mut buffer = BytesMut::new();
        buffer.put_u8(36);
        buffer.put(bytes);

        let uuid = Uuid::from_str("8b9fe9c4-d64e-4c2c-aa21-0c0cdb829253").unwrap();
        assert_eq!(uuid, StringUuid::read(&mut buffer).unwrap().into());
    }

    #[test]
    fn test_str_write() {
        let mut buffer = BytesMut::new();
        let uuid: StringUuid = Uuid::from_str("8b9fe9c4-d64e-4c2c-aa21-0c0cdb829253").unwrap().into();
        uuid.write(&mut buffer).unwrap();

        assert_eq!(36, buffer.get_u8());
        assert_eq!(b"8b9fe9c4-d64e-4c2c-aa21-0c0cdb829253", buffer.as_ref());
    }
}
