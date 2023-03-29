use bytes::BufMut;

use crate::WriteError;

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
    B: BufMut,
{
    if buffer.remaining_mut() < bytes.len() {
        return Err(WriteError::EndOfBuffer);
    }
    buffer.put_slice(bytes);
    Ok(())
}
