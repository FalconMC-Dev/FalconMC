use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
    #[error("Couldn't serialize to NBT")]
    FastNbtError(#[from] fastnbt::error::Error),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Invalid UTF-8 received")]
    FromUTF8Error(#[from] FromUtf8Error),
    #[error("Invalid StrUuid received")]
    UuidError(#[from] uuid::Error),
    #[error("Couldn't deserialize from NBT")]
    FastNbtError(#[from] fastnbt::error::Error),
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
    #[error("VarInt was longer than allowed")]
    VarTooLong,
    #[error("Reached EOF of input buffer")]
    NoMoreBytes,
}
