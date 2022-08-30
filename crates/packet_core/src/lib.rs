use bytes::{Buf, BufMut};

use error::ReadError;

use self::error::WriteError;

pub mod error;

pub trait PacketRead {
    fn from_buf<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized;
}

pub trait PacketWrite {
    fn to_buf<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

pub trait PacketReadSeed {
    type Value;

    fn from_buf<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: Buf + ?Sized;
}

pub trait PacketWriteSeed {
    type Value;

    fn to_buf<B>(self, value: Self::Value, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}
