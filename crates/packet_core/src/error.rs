use std::string::FromUtf8Error;

use thiserror::Error;

/// Errors that can occur when writing types to a minecraft connection using
/// [`PacketWrite`](super::PacketWrite) or
/// [`PacketWriteSeed`](super::PacketWriteSeed).
#[derive(Debug, Error)]
pub enum WriteError {
    /// Returned when a string was longer than the maximum length allowed by the
    /// protocol.
    ///
    /// The first argument is what was attempted to write, the
    /// second argument is the maximum length.
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
    /// Returned by [`fastnbt`](fastnbt).
    #[error("Couldn't serialize to NBT")]
    FastNbtError(#[from] fastnbt::error::Error),
    /// Returned when there is unsufficient space in the supplied
    /// [`BufMut`](bytes::BufMut).
    #[error("Buffer ran out of space")]
    EndOfBuffer,
}

/// Errors that can occur when reading types from a minecraft connection using
/// [`PacketRead`](super::PacketRead) or
/// [`PacketReadSeed`](super::PacketReadSeed).
#[derive(Debug, Error)]
pub enum ReadError {
    /// Returned when parsing UTF-8 fails.
    #[error("Invalid UTF-8 received")]
    FromUTF8Error(#[from] FromUtf8Error),
    /// Returned when parsing [`Uuid`](uuid::Uuid) in invalid strinng
    /// representation.
    #[error("Invalid StrUuid received")]
    UuidError(#[from] uuid::Error),
    /// Returned by [`fastnbt`](fastnbt).
    #[error("Couldn't deserialize from NBT")]
    FastNbtError(#[from] fastnbt::error::Error),
    /// Returned when a string was longer than the maximum length allowed by the
    /// protocol.
    ///
    /// The first argument is the length that was read, the
    /// second argument is the maximum length.
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
    /// Returned when a [`VarI32`](super::VarI32) is larger in size than 5 bytes
    /// or when a [`VarI64`](super::VarI64) is larger in size than 10 bytes.
    #[error("VarInt was longer than allowed")]
    VarTooLong,
    /// Returned when there are unsufficient bytes remaining in
    /// [`Buf`](bytes::Buf).
    #[error("Reached EOF of input buffer")]
    NoMoreBytes,
}
