use thiserror::Error;
use falcon_core::error::FalconCoreError;

pub type Result<T> = std::result::Result<T, DefaultProtocolError>;

#[derive(Error, Debug)]
pub enum DefaultProtocolError {
    #[error("Falcon core threw error")]
    FalconCore(#[from] FalconCoreError)
}
