use thiserror::Error;

use crate::world::blocks::ParseBlockError;

pub type Result<T> = std::result::Result<T, FalconCoreError>;

#[derive(Error, Debug)]
pub enum FalconCoreError {
    #[error("Error while doing IO")]
    IoError(#[from] std::io::Error),
    #[error("The packet length was longer than 21 bits")]
    PacketTooLong,
    #[error("Schematic version {0} is not supported")]
    InvalidSchematic(i32),
    #[error("Invalid schematic data version, should be {0} instead of {1}")]
    WrongDataVersion(i32, i32),
    #[error("Invalid schematic data was found: {0}")]
    InvalidData(String),
    #[error("Could not find the correct schematic blockdata")]
    MissingData,
    #[error("Error while reading block data")]
    ParseBlockError(#[from] ParseBlockError),
    #[error("Tracing level cannot be: {0}")]
    ConfigInvalidTracingLevel(String),
}
