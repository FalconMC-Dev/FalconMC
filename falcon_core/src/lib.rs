// #![warn(missing_docs)]

//! Core crate of FalconMC.
//!
//! Every other crate may use this one as a dependency.
//! This crate shall never depend on another falcon-related
//! crate and hence functions as base layer.
//!
//! This crate contains mostly utility data structures
//! and quality of life functions to quickly extend
//! falconmc's functionality.

mod shutdown;

pub mod config;
pub mod network;
pub mod player;

pub use shutdown::ShutdownHandle;
