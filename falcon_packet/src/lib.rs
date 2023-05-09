//! ## **Falcon Packet Core**
//! This is the main component of [FalconMC](https://github.com/FalconMC-Dev/FalconMC)'s
//! networking system. It defines how types should be read from and written to a
//! minecraft network connection.
//!
//! The design philosophy of this crate is inspired by [serde](https://serde.rs).
//!
//! ## **Traits**
//! Five traits are introduced:
//! - [`PacketRead`]: How to read a type from the network.
//! - [`PacketWrite`]: How to write a type to the network.
//! - [`PacketSize`]: Memory-efficient size computation of the data when it
//!   would be written to the netork.
//! - [`PacketReadSeed`]: How to read a type from the network. This trait is
//!   used to pass data to the read implemenetation.
//! - [`PacketWriteSeed`]: How to write a type to the network. This trait is
//!   used to pass data to the write implemenetation.
//!
//! Because [Minecraft's protocol](https://wiki.vg/) doesn't
//! translate one-to-one to Rust types, this crate offers some
//! convenient wrapper structs to correctly serialize
//! data such as strings or length-prefixed lists to the network.
//!
//! Some of these wrappers also help in reading
//! from the network while maintaining high memory efficiency
//! by leveraging the use of [`bytes::Bytes`].

use bytes::{Buf, BufMut};
pub use error::{ReadError, WriteError};
pub use macro_traits::*;
pub use pub_macro::*;

mod error;
pub mod primitives;
#[rustfmt::skip]
mod pub_macro;
mod macro_traits;

/// A data structure that can be read from a minecraft connection.
pub trait PacketRead {
    /// This function extracts the type from the given buffer.
    ///
    /// # Safety
    /// If this function errors, the buffer is to
    /// be considered **corrupt**. We don't mean Undefined
    /// Behavior here but rather an impossibility to
    /// correctly get the next types from the buffer.
    ///
    /// # Important
    /// Implementations that read directly from the buffer
    /// (no redirection of this function/trait) **must ensure**
    /// that the remaining length of the buffer is always
    /// checked first before reading bytes from it.
    /// This is to eliminate panics.
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized;
}

/// A data structure that can be written to a minecraft connection.
pub trait PacketWrite: PacketSize {
    /// This function serializes the type to the given buffer.
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

/// A data structure that can efficiently compute
/// its serialized size on the network buffer.
pub trait PacketSize {
    /// This function computes the exact network
    /// size of the type.
    ///
    /// # Implementors
    /// It is highly encouraged to optimize this function.
    /// Avoid writing the type to a buffer and returning
    /// that buffer's change in length at all costs.
    fn size(&self) -> usize;
}

/// A data structure that can read another data type from a minecraft
/// connection, see [`DeserializeSeed`](https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html).
///
/// This trait should rarely be implemented manually, if you implement this for
/// a general type, please contribute it to this project.
pub trait PacketReadSeed<T> {
    /// This function extracts the type from the given buffer.
    ///
    /// # Safety
    /// If this function errors, the buffer is to
    /// be considered **corrupt**. We don't mean Undefined
    /// Behavior here but rather an impossibility to
    /// correctly get the next types from the buffer.
    ///
    /// # Important
    /// Implementations that read directly from the buffer
    /// (no redirection of this function/trait) **must ensure**
    /// that the remaining length of the buffer is always
    /// checked first before reading bytes from it.
    /// This is to eliminate panics.
    fn read<B>(self, buffer: &mut B) -> Result<T, ReadError>
    where
        B: Buf + ?Sized;
}

/// A data structure that can write another data type from a minecraft
/// connection, see [`SeralizeSeed`](https://docs.rs/serde/latest/serde/ser/trait.SerializeSeed.html).
///
/// This trait should rarely be implemented manually, if you implement this for
/// a general type, please contribute it to this project.
pub trait PacketWriteSeed<T>
where
    T: PacketSize + ?Sized,
{
    /// This function serializes the type to the given buffer.
    fn write<'a, B>(self, value: &'a T, buffer: &'a mut B) -> Result<(), WriteError>
    where
        B: BufMut;
}
