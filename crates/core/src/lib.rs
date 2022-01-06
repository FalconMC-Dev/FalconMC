extern crate self as falcon_core;

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate tracing;

pub mod errors;
pub mod network;
pub mod world;
pub mod schematic;
pub mod server;
pub mod player;

mod shutdown;

pub use shutdown::ShutdownHandle;
