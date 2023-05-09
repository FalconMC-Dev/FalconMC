use bytes::BufMut;

use super::VarI32;
use crate::{PacketSize, PacketWrite, WriteError};

/// Utility function to quickly write a slice of bytes to
/// a given buffer.
///
/// # Performance
/// This method is the most efficient way to just write
/// a slice of bytes to the network.
/// *Prefer using this over other methods!*
#[inline]
pub fn write_bytes<B>(buffer: &mut B, bytes: &[u8]) -> Result<(), WriteError>
where
    B: BufMut + ?Sized,
{
    if buffer.remaining_mut() < bytes.len() {
        return Err(WriteError::EndOfBuffer);
    }
    buffer.put_slice(bytes);
    Ok(())
}

/// Utility function to quickly write a string slice
/// to a given buffer.
///
/// # Note
/// Unlike [`write_str_unchecked`], this function ensures that
/// the length of the given string is within bounds [0, max]
/// before writing it to the network. This function may thus
/// return a [`StringTooLong`](WriteError::StringTooLong) error.
///
/// # Performance
/// This method is the second most efficient way to
/// write a string slice to the network. See [`write_str_unchecked`].
#[inline]
pub fn write_str<B>(buffer: &mut B, max: usize, str: &str) -> Result<(), WriteError>
where
    B: BufMut + ?Sized,
{
    let count = str.chars().count();
    if count > max {
        Err(WriteError::StringTooLong(max, count))
    } else {
        VarI32::from(str.as_bytes().len()).write(buffer)?;
        write_bytes(buffer, str.as_bytes())
    }
}

/// Unchecked version [`write_str`].
///
/// # Caution
/// This function does not check the length of the string
/// before writing to the network. Always make sure
/// the length of the string is within bounds of the one specified
/// by the protocol. Clients may crash otherwise.
/// (=> this function will never return
/// [`StringTooLong`](WriteError::StringTooLong))
///
/// # Performance
/// This method is the most efficient way to
/// write a string slice to the network. See [`write_str`].
#[inline]
pub fn write_str_unchecked<B>(buffer: &mut B, str: &str) -> Result<(), WriteError>
where
    B: BufMut + ?Sized,
{
    VarI32::from(str.as_bytes().len()).write(buffer)?;
    write_bytes(buffer, str.as_bytes())
}

/// Utility function to compute the size of
/// any iterator of [`PacketSize`]'s.
///
/// # Usage
/// While this function is very general and not necessarily
/// inefficient, types such as slices of bytes can be computed
/// a lot faster than iterating over the slice and counting bytes
/// one by one.
///
/// So please consider more concrete implementations over the
/// use of this function first.
pub fn iter_size<'a, 'b, I, T>(iterator: I) -> usize
where
    'b: 'a,
    T: PacketSize + 'b,
    I: Iterator<Item = &'a T>,
{
    iterator.map(|element| element.size()).sum()
}

/// Utility function to write any iterator of
/// [`PacketWrite`]'s to the network.
///
/// # Usage
/// While this function is very general and not necessarily
/// inefficient, types such as slices of bytes can be written
/// a lot faster than going over every byte separately.
///
/// So please consider more concrete implementations over the
/// use of this function first.
pub fn iter_write<'a, 'b, I, T, B>(mut iterator: I, buffer: &mut B) -> Result<(), WriteError>
where
    'b: 'a,
    B: BufMut + ?Sized,
    T: PacketWrite + 'b,
    I: Iterator<Item = &'a T>,
{
    iterator.try_for_each(|element| element.write(buffer))
}
