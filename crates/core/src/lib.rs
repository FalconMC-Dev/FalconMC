extern crate self as falcon_core;

#[macro_use]
extern crate log;

pub mod errors;
pub mod network;
pub mod world;
pub mod schematic;
pub mod server;

mod shutdown;

pub use shutdown::ShutdownHandle;

pub static LOG_CONFIG: &'static str = "./config/log4rs.yml";
