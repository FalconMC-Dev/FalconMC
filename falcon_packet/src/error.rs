use std::string::FromUtf8Error;

use thiserror::Error;

/// Error variants that may be returned by [`crate::PacketWrite`].
#[derive(Debug, Error)]
pub enum WriteError {
    /// Indication that a given string was longer than allowed
    /// by the protocol.
    ///
    /// The first `usize` is the current length in bytes, the second
    /// `usize` is the maximum allowed length in bytes.
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
    /// Indication that a given struct couldn't be serialized
    /// by [`fastnbt`].
    #[error("Couldn't serialize to NBT")]
    FastNbtError(#[from] fastnbt::error::Error),
    /// Indication there was no more space
    /// left in the buffer when trying to write.
    #[error("Buffer ran out of space")]
    EndOfBuffer,
}

/// Error variants that may be returned by [`crate::PacketRead`].
#[derive(Debug, Error)]
pub enum ReadError {
    /// Indication that the read string was invalid utf8.
    #[error("Invalid UTF-8 received")]
    FromUTF8Error(#[from] FromUtf8Error),
    /// Indication that the read uuid was invalid.
    #[error("Invalid StrUuid received")]
    UuidError(#[from] uuid::Error),
    /// Indication that the target struct could not
    /// be deserialized by [`fastnbt`].
    #[error("Couldn't deserialize from NBT")]
    FastNbtError(#[from] fastnbt::error::Error),
    /// Indication that the read string was longer than
    /// allowed by the protocol.
    ///
    /// The first `usize` is the actual length in bytes, the
    /// second `usize` is the maximum allowed length in bytes.
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
    /// Indication that the received variable-length integer was
    /// longer than allowed. (32 or 64 bit)
    #[error("VarInt was longer than allowed")]
    VarTooLong,
    /// Indication that the given buffer has no more bytes
    /// left to read from.
    #[error("Reached EOF of input buffer")]
    NoMoreBytes,
}
