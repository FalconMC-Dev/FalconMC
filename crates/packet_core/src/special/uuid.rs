use std::ops::Deref;
use std::str::FromStr;

use bytes::{Buf, BufMut};
use uuid::Uuid;

use crate::error::{ReadError, WriteError};
use crate::{PacketRead, PacketReadSeed, PacketSize, PacketString, PacketWrite, PacketWriteSeed};

impl PacketWrite for Uuid {
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.as_ref().write(buffer)
    }
}

impl PacketSize for Uuid {
    #[inline]
    fn size(&self) -> usize { 16 }
}

impl PacketRead for Uuid {
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized,
    {
        let bytes = <[u8; 16]>::read(buffer)?;
        Ok(Uuid::from_bytes(bytes))
    }
}

/// String representation of [`Uuid`] as a wrapper.
///
/// Instance creation should happen using [`From`].
///
/// Unlike normal strings, the string representation of a uuid always has a
/// maximum length of 36. This allows this type to implement [`PacketRead`],
/// [`PacketSize`] and [`PacketWrite`] directly instead of the seed-flavored
/// traits.
///
/// # Examples
/// Writing a `StrUuid`:
/// ```
/// use uuid::Uuid;
/// use bytes::BytesMut;
/// use falcon_packet_core::special::StrUuid;
/// use falcon_packet_core::{PacketSize, PacketWrite};
///
/// let mut buffer = BytesMut::new();
///
/// let uuid = Uuid::parse_str("ec174daf-b5a5-4ea1-adc6-35a7f9fc4a60").unwrap(); // random uuid
/// let str_uuid = StrUuid::from(uuid);
///
/// assert_eq!(str_uuid.size(), 37);
///
/// str_uuid.write(&mut buffer)?;
/// assert_eq!(buffer.len(), 37); // assert a minecraft string of length 36 has been written
/// # Ok::<(), falcon_packet_core::WriteError>(())
/// ```
///
/// Reading a `StrUuid`:
/// ```
/// use uuid::Uuid;
/// use bytes::{BytesMut, BufMut};
/// use falcon_packet_core::special::StrUuid;
/// use falcon_packet_core::PacketRead;
///
/// // buffer containing a random uuid
/// let mut buffer = BytesMut::new();
/// buffer.put_u8(36);
/// buffer.put_slice(b"ec174daf-b5a5-4ea1-adc6-35a7f9fc4a60");
///
/// let str_uuid = StrUuid::read(&mut buffer)?;
/// let uuid = Uuid::from(str_uuid);
///
/// assert_eq!(uuid, Uuid::parse_str("ec174daf-b5a5-4ea1-adc6-35a7f9fc4a60").unwrap());
/// # Ok::<(), falcon_packet_core::ReadError>(())
/// ```
pub struct StrUuid(pub(crate) Uuid);

const STR_UUID_LEN: usize = {
    // copied over from VarI32
    let n = ({ i32::BITS as usize + 6 } - (uuid::fmt::Hyphenated::LENGTH as i32).leading_zeros() as usize) / 7;
    n + uuid::fmt::Hyphenated::LENGTH
};

impl From<Uuid> for StrUuid {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<StrUuid> for Uuid {
    fn from(id: StrUuid) -> Self { id.0 }
}

impl PacketWrite for StrUuid {
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        let mut buf = [0u8; uuid::fmt::Hyphenated::LENGTH];
        PacketWriteSeed::write(PacketString::new(uuid::fmt::Hyphenated::LENGTH), &self.0.hyphenated().encode_lower(&mut buf), buffer)
    }
}

impl PacketSize for StrUuid {
    fn size(&self) -> usize { STR_UUID_LEN }
}

impl PacketRead for StrUuid {
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized,
    {
        let s: String = PacketString::new(36).read(buffer)?;
        Ok(Self(Uuid::from_str(s.deref())?))
    }
}
