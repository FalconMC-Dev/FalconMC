#![doc(html_favicon_url = "https://wiki.falconmc.org/perm/icons/favicon.ico?v=1")]
#![doc(html_logo_url = "https://wiki.falconmc.org/perm/icons/android-chrome-512x512.png?v=1")]
//! ## **Packet Core**
//! This is the main component of [Falcon](https://github.com/FalconMC-Dev/FalconMC)'s
//! networking system. It defines how types should be read from and written to a
//! network connection.
//!
//! The design philosophy of this crate is inspired by [serde](https://serde.rs).
//!
//! ## **Traits**
//! Six traits are introduced:
//! - [`PacketRead`]: A data structure that can be read **without** the need for
//!   external input.
//! - [`PacketWrite`]: A data structure that can be written **without** the need
//!   for external input.
//! - [`PacketSize`]: Used to determine the size in bytes of the byte array that
//!   `PacketWrite` would produce.
//! - [`PacketReadSeed`]: A data structure that can be read but **with** a need
//!   for external input.
//! - [`PacketWriteSeed`]: A data structure that can be written but **with** a
//!   need for external input.
//! - [`PacketSizeSeed`]: Used to determine the size in bytes of the byte array
//!   that
//! `PacketWriteSeed` would produce.
//!
//! Examples of types that require external input are:
//! - **Strings**: The minecraft protocol specifies a maximum length for every
//!   string field, this
//! value is checked when reading and writing.
//! - **Byte arrays**: Unlike strings, byte arrays are not prefixed by their
//!   length, so these
//! generally need to know their length from some other field when reading.
//! - ...
//!
//! ## **How to implement**
//! For user implementations, it is highly encouraged to use the following
//! derive macros:
//! - [`PacketRead`](falcon_packet_core_derive::PacketRead)
//! - [`PacketWrite`](falcon_packet_core_derive::PacketWrite)
//! - [`PacketSize`](falcon_packet_core_derive::PacketSize)
//!
//! ## **Provided implementations**:
//! Numerous implementations are provided for most basic types:
//! - **Primitive types**:
//!     - bool
//!     - i8, i16, i32, i64, i128
//!     - u8, u16, u32, u64, u128
//!     - f32, f64
//! - **Arrays**:
//!     - [T; N] for all N
//!     - [u8; N] for all N
//! - **String**:
//!     - AsRef<str> for writing
//!     - From<String> for reading
//! - **Byte sequences**:
//!     - AsRef<\[u8]> for writing
//!     - From<Vec\<u8>> for reading
//! - **Iterators**
//!     - IntoIterator for writing
//!     - FromIterator for reading
//! - **Extra**
//!     - Uuid
//!     - StrUuid (string representation of uuid)

extern crate self as falcon_packet_core;
use bytes::{Buf, BufMut};
pub use error::{ReadError, WriteError};
pub use falcon_packet_core_derive::{PacketRead, PacketSize, PacketWrite};
pub use primitives::*;

mod error;
pub mod special;
mod test;

mod primitives;

/// A data structure that can be read from a minecraft connection without
/// needing external input; aimed to be highly modular.
///
/// Users should aim to avoid implementing this trait directly, use the provided
/// [derive macros].
///
/// [derive macros]: falcon_packet_core#derives
pub trait PacketRead {
    /// Read self from the buffer according to the minecraft protocol.
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized;
}

/// A data structure that can be written to a minecraft connection without
/// needing external input; aimed to be highly modular.
///
/// Users should aim to avoid implementing this trait directly, use the provided
/// [derive macros].
///
/// [derive macros]: falcon_packet_core#derives
pub trait PacketWrite: PacketSize {
    /// Write self to the buffer according to the minecraft protocol.
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

/// A data structure that can determine how large it will be in bytes when
/// written to a minecraft connection without needing external input; aimed to
/// be highly modular.
///
/// Users should aim to avoid implementing this trait directly, use the provided
/// [derive macros].
///
/// [derive macros]: falcon_packet_core#derives
pub trait PacketSize {
    /// Determine the size Self would have if it was written to
    /// a minecraft connection, this should be an exact value.
    fn size(&self) -> usize;
}

/// A data structure that can read another data type from a minecraft
/// connection. The implementing type usually stores a length or similar data.
/// Examples from this crate include implementing read for all types that
/// implement [`From<Vec<u8>>`].
///
/// This trait should rarely be implemented manually, if you implement this for
/// a general type, please contribute it to this project.
pub trait PacketReadSeed {
    /// The target type this struct will produce.
    type Value;

    /// Read [`Self::Value`](PacketReadSeed::Value) from the buffer using self,
    /// according to the minecraft protocol.
    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: Buf + ?Sized;
}

/// A data structure that can write another data type from a minecraft
/// connection. The implementing type usually stores a length or similar data.
/// Examples from this crate include implementing write for all types that
/// implement [`AsRef<[u8]>`](std::convert::AsRef]>`).
///
/// This trait should rarely be implemented manually, if you implement this for
/// a general type, please contribute it to this project.
pub trait PacketWriteSeed<'a>: PacketSizeSeed<'a> {
    /// Write [`Self::Value`](PacketSizeSeed::Value) to the buffer using self,
    /// according to the minecraft protocol.
    fn write<B>(self, value: &'a Self::Value, buffer: &'a mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized;
}

/// A data structure that can determine the size of the data written by
/// [`PacketWriteSeed`]. The implementing type usually stores a length or
/// similar data. Examples from the crate include implementing size for all
/// types that implement [`AsRef<[u8]>`](std::convert::AsRef]>`).
///
/// This trait should rarely be implemented manually, if you implement this for
/// a general type, please contribute it to this project.
pub trait PacketSizeSeed<'a> {
    /// The target type this struct will write.
    type Value;

    /// Determine the size [`Self::Value`](PacketReadSeed::Value) would have if
    /// it was written to a minecraft connection, this should be an exact
    /// value.
    fn size(self, value: &'a Self::Value) -> usize;
}
