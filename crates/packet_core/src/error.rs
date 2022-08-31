use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("IO write error")]
    IoError(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("IO read error")]
    IoError(#[from] io::Error),
    #[error("VarInt was longer than allowed")]
    VarTooLong,
}
