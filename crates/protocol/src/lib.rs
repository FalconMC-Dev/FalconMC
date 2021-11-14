#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use std::alloc::System;
use std::any::Any;

use errors::*;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;

pub mod errors;
mod macros;
pub mod manager;

#[global_allocator]
static ALLOCATOR: System = System;

pub static UNKNOWN_PROTOCOL: i32 = -1;

pub trait ProtocolPlugin: Any + Send + Sync {
    fn name(&self) -> &'static str;

    fn on_protocol_load(&self) {}

    fn on_protocol_unload(&self) {}

    /// Return the importance of this `ProtocolPlugin`'s packet querying, lower numbers are more important.
    ///
    /// 0-1-2-3 are reserved for default implementations.
    fn get_priority(&self) -> i32 {
        4
    }

    fn process_packet(
        &self,
        packet_id: i32,
        buffer: &mut dyn PacketBufferRead,
        connection: &mut dyn MinecraftConnection,
    ) -> Option<Result<()>>;
}
