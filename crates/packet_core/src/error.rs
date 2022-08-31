use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("VarInt was longer than allowed")]
    VarTooLong,
}
