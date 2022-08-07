#[macro_use]
extern crate tracing;

mod macros;

packet_modules! {
    extern pub mod v1_8_9;
    extern pub mod v1_12_2;
    extern pub mod v1_9;
}
