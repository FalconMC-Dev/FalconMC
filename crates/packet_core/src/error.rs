use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Invalid UTF-8 received")]
    FromUTF8Error(#[from] FromUtf8Error),
    #[error("String was longer than allowed: {1} > {0}")]
    StringTooLong(usize, usize),
    #[error("VarInt was longer than allowed")]
    VarTooLong,
}
