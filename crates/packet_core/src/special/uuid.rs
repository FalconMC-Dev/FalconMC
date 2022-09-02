use std::ops::Deref;
use std::str::FromStr;

use bytes::{Buf, BufMut};
use uuid::Uuid;

use crate::error::{ReadError, WriteError};
use crate::{PacketRead, PacketReadSeed, PacketSize, PacketString, PacketWrite, PacketWriteSeed};

impl PacketWrite for Uuid {
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        self.as_ref().write(buffer)
    }
}

impl PacketSize for Uuid {
    #[inline]
    fn size(&self) -> usize {
        16
    }
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

pub struct StrUuid(pub(crate) Uuid);

const STR_UUID_LEN: usize = {
    // copied over from VarI32
    let n = ({ i32::BITS as usize + 6 }
        - (uuid::fmt::Hyphenated::LENGTH as i32).leading_zeros() as usize)
        / 7;
    n + uuid::fmt::Hyphenated::LENGTH
};

impl From<Uuid> for StrUuid {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<StrUuid> for Uuid {
    fn from(id: StrUuid) -> Self {
        id.0
    }
}

impl PacketWrite for StrUuid {
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        let mut buf = [0u8; uuid::fmt::Hyphenated::LENGTH];
        PacketString::new(uuid::fmt::Hyphenated::LENGTH)
            .write(self.0.hyphenated().encode_lower(&mut buf), buffer)
    }
}

impl PacketSize for StrUuid {
    fn size(&self) -> usize {
        STR_UUID_LEN
    }
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
