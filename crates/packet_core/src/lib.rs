use bytes::{Buf, BufMut};

use error::ReadError;

use self::error::WriteError;

pub use primitives::*;

pub mod error;
mod primitives;

pub trait PacketRead {
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized;
}

pub trait PacketWrite: PacketSize {
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

pub trait PacketReadSeed {
    type Value;

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: Buf + ?Sized;
}

pub trait PacketWriteSeed {
    type Value;

    fn write<B>(self, value: Self::Value, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

pub trait PacketSize {
    fn size(&self) -> usize;
}
