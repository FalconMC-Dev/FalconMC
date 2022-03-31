use thiserror::Error;
use falcon_core::error::FalconCoreError;

pub type Result<T> = std::result::Result<T, PluginProtocolError>;

#[derive(Error, Debug)]
pub enum PluginProtocolError {
    #[error("Falcon core threw an error")]
    FalconCore(#[from] FalconCoreError),
    #[error("Could not load library {0:?} due to {1}")]
    LibraryLoadingError(::std::ffi::OsString, String),
}
