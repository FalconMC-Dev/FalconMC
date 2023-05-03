use downcast_rs::{impl_downcast, Downcast};
mod pub_macro;
pub use pub_macro::*;

extern crate self as falcon_protocol;

/// Marker trait for packets.
///
/// Used in combination with [`downcast_rs`](https://crates.io/crates/downcast-rs)
/// to obtain individual packets defined in this crate.
pub trait Packet: Downcast {}
impl_downcast!(Packet);
