use std::borrow::Cow;

use bytes::Bytes;
use castaway::cast;

use super::{write_str, VarI32};
use crate::{PacketRead, PacketReadSeed, PacketSize, PacketWrite, PacketWriteSeed, ReadError, WriteError};

impl<T> PacketWriteSeed<T> for usize
where
    T: AsRef<str> + PacketSize + 'static,
{
    fn write<'a, B>(self, value: &'a T, buffer: &'a mut B) -> Result<(), WriteError>
    where
        B: bytes::BufMut,
    {
        // Specizliation for PacketString
        if let Ok(packet_str) = cast!(value, &PacketString) {
            #[cfg(all(test, feature = "verbose-test"))]
            eprintln!("Using PacketString impl");
            let count = packet_str.as_ref().chars().count();
            if count > self {
                Err(WriteError::StringTooLong(self, count))
            } else {
                VarI32::from(packet_str.as_bytes().len()).write(buffer)?;
                super::write_bytes(buffer, packet_str.as_bytes())
            }
        } else {
            self.write(value.as_ref(), buffer)
        }
    }
}

/// `self` acts as a maximum length here.
impl PacketWriteSeed<str> for usize {
    fn write<'a, B>(self, value: &'a str, buffer: &'a mut B) -> Result<(), WriteError>
    where
        B: bytes::BufMut,
    {
        write_str(buffer, self, value)
    }
}

impl PacketSize for str {
    fn size(&self) -> usize { VarI32::from(self.len()).size() + self.len() }
}

/// A specialized string wrapper for the minecraft protocol.
///
/// This type provides an efficient implementation for reading
/// and writing strings from the network.
/// To explicitly encourage developers to use this
/// type for performance, there are no packet implementations
/// for [`String`] and [`Cow<'a, str>`](Cow) provided.
///
/// # Invariant
///
/// The bytes inside the [`Slice`](PacketString::Slice) variant
/// must always be valid utf-8. If this is not upheld, this struct
/// is allowed to **crash the program**.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PacketString {
    Owned(String),
    /// Cow buffer of sorts, used when reading a string from the network.
    /// This type allows for better performance when the string isn't used
    /// after reading.
    ///
    /// # Important
    /// Must always contain valid utf-8!!
    Slice(Bytes),
}

impl PacketString {
    pub fn new() -> Self { Default::default() }

    pub fn from_static(str: &'static str) -> Self { PacketString::Slice(Bytes::from_static(str.as_bytes())) }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            PacketString::Owned(v) => v.as_bytes(),
            PacketString::Slice(v) => v.as_ref(),
        }
    }

    /// Returns the length of the string in bytes
    pub fn len(&self) -> usize {
        match self {
            PacketString::Owned(v) => v.len(),
            PacketString::Slice(v) => v.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

impl Default for PacketString {
    fn default() -> Self { Self::Slice(Default::default()) }
}

impl<'a> From<&'a str> for PacketString {
    fn from(value: &'a str) -> Self { Self::Slice(Bytes::copy_from_slice(value.as_bytes())) }
}

impl From<String> for PacketString {
    fn from(value: String) -> Self { Self::Owned(value) }
}

impl<'a> From<Cow<'a, str>> for PacketString {
    fn from(value: Cow<'a, str>) -> Self { Self::Slice(Bytes::copy_from_slice(value.as_bytes())) }
}

impl AsRef<str> for PacketString {
    fn as_ref(&self) -> &str {
        match self {
            PacketString::Owned(v) => v.as_ref(),
            // SAFETY: We assume there is valid utf-8 in the byte buffer (invariant of the type)
            PacketString::Slice(v) => unsafe { std::str::from_utf8_unchecked(v.as_ref()) },
        }
    }
}

impl From<PacketString> for String {
    fn from(value: PacketString) -> Self {
        match value {
            PacketString::Owned(v) => v,
            // SAFETY: We assume there is valid utf-8 in the byte buffer (invariant of the type)
            PacketString::Slice(v) => String::from(unsafe { std::str::from_utf8_unchecked(v.as_ref()) }),
        }
    }
}

/// `self` acts as a maximum length here.
impl PacketReadSeed<PacketString> for usize {
    fn read<B>(self, buffer: &mut B) -> Result<PacketString, crate::ReadError>
    where
        B: bytes::Buf + ?Sized,
    {
        let len = VarI32::read(buffer)?.as_usize();
        if len > self * 4 {
            return Err(ReadError::StringTooLong(self * 4, len));
        }
        let bytes: Bytes = len.read(buffer)?;
        let str = std::str::from_utf8(bytes.as_ref())?;
        let count = str.chars().count();
        if count > self {
            Err(ReadError::StringTooLong(self, count))
        } else {
            Ok(PacketString::Slice(bytes))
        }
    }
}

impl PacketSize for PacketString {
    fn size(&self) -> usize { VarI32::from(self.len()).size() + self.len() }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_read() {
        let mut buffer = Bytes::from_static(&[0x06, 0x4e, 0x6f, 0x74, 0x63, 0x68]);
        assert_eq!(Err(ReadError::NoMoreBytes), PacketReadSeed::<PacketString>::read(10, &mut buffer)); // fails because there's not 6, but 5 bytes
        let mut buffer = Bytes::from_static(&[0x06, 0x4e, 0x6f, 0x74, 0x63, 0x68]);
        assert_eq!(Err(ReadError::NoMoreBytes), PacketReadSeed::<PacketString>::read(3, &mut buffer)); // fails because 6 < 3 * 4 but there's still 5 bytes instead of 6
        let mut buffer = Bytes::from_static(&[0x05, 0x4e, 0x6f, 0x74, 0x63, 0x68]);
        assert_eq!(Err(ReadError::StringTooLong(3, 5)), PacketReadSeed::<PacketString>::read(3, &mut buffer)); // fails because while 6 < 3 * 4, there's 5 > 3 bytes
        let mut buffer = Bytes::from_static(&[0x05, 0x4e, 0x6f, 0x74, 0x63, 0x68]);
        assert_eq!("Notch", PacketReadSeed::<PacketString>::read(5, &mut buffer).unwrap().as_ref());
        let mut buffer = Bytes::from_static(&[0x05, 0x4e, 0x6f, 0x74, 0x63, 0x68, 0x03, 0x4a, 0x65, 0x62]);
        assert_eq!("Notch", PacketReadSeed::<PacketString>::read(5, &mut buffer).unwrap().as_ref());
        assert_eq!("Jeb", PacketReadSeed::<PacketString>::read(4, &mut buffer).unwrap().as_ref());
    }

    #[test]
    fn test_write() {
        let mut buffer = BytesMut::new().limit(3);
        let str = PacketString::from("Jeb");
        assert_eq!(Err(WriteError::EndOfBuffer), 4.write(&str, &mut buffer));
        let mut buffer = BytesMut::new();
        4.write(&str, &mut buffer).unwrap();
        assert_eq!(&[0x03, 0x4a, 0x65, 0x62], buffer.as_ref());
        let notch = PacketString::from("Notch");
        assert_eq!(Err(WriteError::StringTooLong(4, 5)), 4.write(&notch, &mut buffer));
        5.write(&notch, &mut buffer).unwrap();
        assert_eq!(&[0x03, 0x4a, 0x65, 0x62, 0x05, 0x4e, 0x6f, 0x74, 0x63, 0x68], buffer.as_ref());
    }
}
