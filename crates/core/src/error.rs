use std::string::FromUtf8Error;
use thiserror::Error;
use crate::world::blocks::ParseBlockError;

pub type Result<T> = std::result::Result<T, FalconCoreError>;

#[derive(Error, Debug)]
pub enum FalconCoreError {
    #[error("PacketBuffer reached EOF")]
    NoMoreBytes,
    #[error("Variable i32 was longer than 5 bytes")]
    VarI32TooLong,
    #[error("Variable i64 was longer than 10 bytes")]
    VarI64TooLong,
    #[error("Invalid utf-8")]
    BadString(#[from] FromUtf8Error),
    #[error("String size was 0")]
    StringSizeZero,
    #[error("String was longer than expected: {1} > {0}")]
    StringTooLong(i32, i32),
    #[error("Schematic version {0} is not supported")]
    InvalidSchematic(i32),
    #[error("Invalid schematic data version, should be {0} instead of {1}")]
    WrongDataVersion(i32, i32),
    #[error("Invalid schematic data was found: {0}")]
    InvalidData(String),
    #[error("Could not find the correct schematic blockdata")]
    MissingData,
    #[error("Error while reading block data")]
    ParseBlockError(#[from] ParseBlockError)
}
