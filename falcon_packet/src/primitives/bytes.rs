use std::borrow::{Borrow, Cow};
use std::ops::Deref;
use std::slice::Iter;

use bytes::{Bytes, BytesMut};

use crate::{PacketReadSeed, PacketSize, PacketWrite, ReadError, WriteError};

impl PacketReadSeed<Bytes> for usize {
    fn read<B>(self, buffer: &mut B) -> Result<Bytes, ReadError>
    where
        B: bytes::Buf + ?Sized,
    {
        if buffer.remaining() < self {
            return Err(ReadError::NoMoreBytes);
        }
        Ok(buffer.copy_to_bytes(self))
    }
}

impl PacketReadSeed<BytesMut> for usize {
    fn read<B>(self, buffer: &mut B) -> Result<BytesMut, ReadError>
    where
        B: bytes::Buf + ?Sized,
    {
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&PacketReadSeed::<Bytes>::read(self, buffer)?);
        Ok(bytes)
    }
}

impl PacketWrite for Bytes {
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: bytes::BufMut,
    {
        if buffer.remaining_mut() < self.len() {
            return Err(WriteError::EndOfBuffer);
        }
        buffer.put(self.clone());
        Ok(())
    }
}

impl PacketWrite for BytesMut {
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: bytes::BufMut,
    {
        if buffer.remaining_mut() < self.len() {
            return Err(WriteError::EndOfBuffer);
        }
        buffer.put(self.clone());
        Ok(())
    }
}

impl PacketSize for Bytes {
    fn size(&self) -> usize { self.len() }
}

impl PacketSize for BytesMut {
    fn size(&self) -> usize { self.len() }
}

/// A specialized byte buffer for the minecraft protocol.
///
/// This provides an efficient implementation for both reading
/// and writing bytes between a [`Vec<u8>`] and a [`Bytes`].
/// Therefore it might be more interesting to use than only a [`Bytes`].
///
/// Always use this type over a plain [`Vec<u8>`].
///
/// # Performance
/// This type is meant as a read-write byte buffer that also **stores
/// the bytes** (meaning it might perform a copy). If you just need
/// to write a slice of bytes to the network, use
/// [`write_bytes`](super::write_bytes). This will
/// avoid unnecessary copies.
///
/// # Note
/// Because byte arrays must be read from a fixed length,
/// this type only implements [`PacketWrite`] and [`PacketSize`].
/// [`PacketReadSeed<PacketBytes>`] is implemented for [`usize`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PacketBytes {
    /// Used for owned buffer, usually used when writing `Vec<u8>` to the
    /// network.
    Vec(Vec<u8>),
    /// Cow buffer of sorts, usually used when reading bytes from the network.
    /// This type allows for better performance when the bytes aren't used.
    Slice(Bytes),
}

impl PacketBytes {
    pub fn new() -> Self { Default::default() }

    pub fn from_static(bytes: &'static [u8]) -> Self { PacketBytes::Slice(Bytes::from_static(bytes)) }
}

impl PacketReadSeed<PacketBytes> for usize {
    fn read<B>(self, buffer: &mut B) -> Result<PacketBytes, ReadError>
    where
        B: bytes::Buf + ?Sized,
    {
        if buffer.remaining() < self {
            return Err(ReadError::NoMoreBytes);
        }
        Ok(buffer.copy_to_bytes(self).into())
    }
}

impl PacketWrite for PacketBytes {
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: bytes::BufMut,
    {
        if buffer.remaining_mut() < self.len() {
            return Err(WriteError::EndOfBuffer);
        }
        match self {
            PacketBytes::Vec(v) => buffer.put_slice(v.as_ref()),
            PacketBytes::Slice(v) => buffer.put(v.clone()),
        }
        Ok(())
    }
}

impl PacketSize for PacketBytes {
    #[inline]
    fn size(&self) -> usize { self.len() }
}

impl Default for PacketBytes {
    fn default() -> Self { Self::Slice(Default::default()) }
}

impl<'a> From<&'a [u8]> for PacketBytes {
    fn from(value: &'a [u8]) -> Self { PacketBytes::Slice(Bytes::copy_from_slice(value)) }
}

impl From<Bytes> for PacketBytes {
    fn from(value: Bytes) -> Self { PacketBytes::Slice(value) }
}

impl From<BytesMut> for PacketBytes {
    fn from(value: BytesMut) -> Self { PacketBytes::Slice(value.into()) }
}

impl From<Vec<u8>> for PacketBytes {
    fn from(value: Vec<u8>) -> Self { PacketBytes::Vec(value) }
}

impl FromIterator<u8> for PacketBytes {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self { PacketBytes::Vec(Vec::from_iter(iter)) }
}

impl<'a> From<Cow<'a, [u8]>> for PacketBytes {
    fn from(value: Cow<'a, [u8]>) -> Self { PacketBytes::Slice(Bytes::copy_from_slice(value.as_ref())) }
}

impl AsRef<[u8]> for PacketBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            PacketBytes::Vec(v) => v.as_slice(),
            PacketBytes::Slice(v) => v.as_ref(),
        }
    }
}

impl Borrow<[u8]> for PacketBytes {
    fn borrow(&self) -> &[u8] {
        match self {
            PacketBytes::Vec(v) => v.borrow(),
            PacketBytes::Slice(v) => v.borrow(),
        }
    }
}

impl Deref for PacketBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            PacketBytes::Vec(v) => v.as_ref(),
            PacketBytes::Slice(v) => v.as_ref(),
        }
    }
}

impl<'a> IntoIterator for &'a PacketBytes {
    type IntoIter = Iter<'a, u8>;
    type Item = &'a u8;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            PacketBytes::Vec(v) => v.iter(),
            PacketBytes::Slice(v) => v.into_iter(),
        }
    }
}

impl IntoIterator for PacketBytes {
    type IntoIter = IntoIter;
    type Item = u8;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            PacketBytes::Vec(v) => IntoIter::Vec(v.into_iter()),
            PacketBytes::Slice(v) => IntoIter::Slice(v.into_iter()),
        }
    }
}

/// [`Iterator`] wrapper for both [`std::vec::IntoIter<u8>`] and
/// [`bytes::buf::IntoIter<Bytes>`].
///
/// Used to iterate over a [`PacketBytes`] instance
/// while consuming ownership.
pub enum IntoIter {
    Vec(std::vec::IntoIter<u8>),
    Slice(bytes::buf::IntoIter<Bytes>),
}

impl Iterator for IntoIter {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Vec(v) => v.next(),
            IntoIter::Slice(v) => v.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            IntoIter::Vec(v) => v.size_hint(),
            IntoIter::Slice(v) => v.size_hint(),
        }
    }

    #[inline]
    fn count(self) -> usize {
        match self {
            IntoIter::Vec(v) => v.count(),
            IntoIter::Slice(v) => v.count(),
        }
    }
}

impl From<PacketBytes> for Vec<u8> {
    fn from(value: PacketBytes) -> Self {
        match value {
            PacketBytes::Vec(v) => v,
            PacketBytes::Slice(v) => v.into(),
        }
    }
}

impl From<PacketBytes> for Bytes {
    fn from(value: PacketBytes) -> Self {
        match value {
            PacketBytes::Vec(v) => Bytes::from(v),
            PacketBytes::Slice(v) => v,
        }
    }
}

impl ExactSizeIterator for IntoIter {}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use super::*;

    #[test]
    fn test_bytes_read() {
        let mut buffer = Bytes::from_static(&[0, 1, 2, 3]);
        assert!(PacketReadSeed::<Bytes>::read(5, &mut buffer).is_err());
        let result: Bytes = 1.read(&mut buffer).unwrap();
        assert_eq!(&[0], result.as_ref());
        let result: Bytes = 3.read(&mut buffer).unwrap();
        assert_eq!(&[1, 2, 3], result.as_ref());
    }

    #[test]
    fn test_bytes_write() {
        let bytes = Bytes::from_static(&[0, 1, 2, 3, 4]);
        assert_eq!(5, bytes.size());
        assert_eq!(5, bytes.len());
        let mut buffer = BytesMut::new().limit(4);
        assert!(bytes.write(&mut buffer).is_err());
        let mut buffer = buffer.into_inner();
        bytes.write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3, 4], buffer.as_ref());

        let bytes = [4, 3, 2, 1];
        let bytes = Bytes::copy_from_slice(&bytes[..]);
        bytes.write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3, 4, 4, 3, 2, 1], buffer.as_ref());
    }

    #[test]
    fn test_packet_read() {
        let mut buffer = Bytes::from_static(&[0, 1, 2, 3]);
        assert!(PacketReadSeed::<PacketBytes>::read(5, &mut buffer).is_err());
        let result: PacketBytes = 1.read(&mut buffer).unwrap();
        assert_eq!(&[0], result.as_ref());
        let result: PacketBytes = 3.read(&mut buffer).unwrap();
        assert_eq!(&[1, 2, 3], result.as_ref());
    }

    #[test]
    fn test_packet_write() {
        let bytes = PacketBytes::from_static(&[0, 1, 2, 3, 4]);
        assert_eq!(5, bytes.size());
        assert_eq!(5, bytes.len());
        let mut buffer = BytesMut::new().limit(4);
        assert!(bytes.write(&mut buffer).is_err());
        let mut buffer = buffer.into_inner();
        bytes.write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3, 4], buffer.as_ref());

        let bytes = [4, 3, 2, 1];
        let bytes = PacketBytes::from(&bytes[..]);
        bytes.write(&mut buffer).unwrap();
        assert_eq!(&[0, 1, 2, 3, 4, 4, 3, 2, 1], buffer.as_ref());
    }

    #[test]
    fn test_iter() {
        let mut buffer = Bytes::from_static(&[0, 1, 2, 3]);
        let result: PacketBytes = 4.read(&mut buffer).unwrap();
        for (check, &i) in result.iter().enumerate() {
            assert_eq!(check as u8, i);
        }
        for (check, i) in result.into_iter().enumerate() {
            assert_eq!(check as u8, i);
        }
    }
}
