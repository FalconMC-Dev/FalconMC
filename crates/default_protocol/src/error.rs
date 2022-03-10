use thiserror::Error;
use falcon_core::error::FalconCoreError;
use falcon_core::network::packet::PacketHandlerError;

pub type Result<T> = std::result::Result<T, DefaultProtocolError>;

#[derive(Error, Debug)]
pub enum DefaultProtocolError {
    #[error("Falcon core threw error")]
    FalconCore(#[from] FalconCoreError),
    #[error("Error while executing packet logic")]
    PacketHandleError(#[from] PacketHandlerError),
}
