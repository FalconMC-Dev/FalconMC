#![macro_use]

use thiserror::Error;
use falcon_core::error::FalconCoreError;
use falcon_default_protocol::error::DefaultProtocolError;

pub type Result<T> = std::result::Result<T, PluginProtocolError>;

#[derive(Error, Debug)]
pub enum PluginProtocolError {
    #[error("Falcon core threw an error")]
    FalconCore(#[from] FalconCoreError),
    #[error("Default protocol threw an error")]
    DefaultProtocol(#[from] DefaultProtocolError),
    #[error("Could not load library {0:?} due to {1}")]
    LibraryLoadingError(::std::ffi::OsString, String),
}
