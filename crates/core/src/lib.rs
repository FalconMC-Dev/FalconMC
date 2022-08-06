extern crate self as falcon_core;

#[macro_use]
extern crate tracing;

pub mod data;
pub mod error;
pub mod network;
pub mod player;
pub mod schematic;
pub mod server;
pub mod world;

mod shutdown;

pub use shutdown::ShutdownHandle;
