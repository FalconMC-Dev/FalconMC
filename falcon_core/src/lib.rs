// #![warn(missing_docs)]

mod shutdown;

pub mod config;
pub mod network;
pub mod player;

pub use shutdown::ShutdownHandle;
