use bytes::{Buf, BufMut};

pub use error::{ReadError, WriteError};

pub use primitives::*;

pub mod error;
pub mod special;

mod primitives;

pub trait PacketRead {
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized;
}

pub trait PacketWrite: PacketSize {
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

pub trait PacketSize {
    fn size(&self) -> usize;
}

pub trait PacketReadSeed {
    type Value;

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: Buf + ?Sized;
}

pub trait PacketWriteSeed<'a>: PacketSizeSeed<'a> {
    fn write<B>(self, value: &'a Self::Value, buffer: &'a mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

pub trait PacketSizeSeed<'a> {
    type Value;

    fn size(self, value: &'a Self::Value) -> usize;
}
