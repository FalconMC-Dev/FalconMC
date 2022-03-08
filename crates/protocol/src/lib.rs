#[macro_use]
extern crate tracing;

use std::alloc::System;
use std::any::Any;

use error::Result;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_default_protocol::DisconnectPacketLogin;

pub use falcon_default_protocol::ProtocolSend;

pub mod error;
mod macros;
pub mod manager;

#[global_allocator]
static ALLOCATOR: System = System;

pub static UNKNOWN_PROTOCOL: i32 = -1;

pub trait FalconPlugin: Any + Send + Sync {
    fn name(&self) -> &'static str;

    fn on_protocol_load(&self) {}

    fn on_protocol_unload(&self) {}

    /// Returns the importance of this `FalconPlugin`'s packet querying, lower numbers are more important.
    ///
    /// 0-1-2-3 are reserved for special implementations.
    fn get_priority(&self) -> i32 {
        4
    }

    fn process_packet(
        &self,
        packet_id: i32,
        buffer: &mut dyn PacketBufferRead,
        connection: &mut dyn MinecraftConnection,
    ) -> Result<Option<()>>;
}

pub fn build_disconnect_packet(reason: String) -> DisconnectPacketLogin {
    DisconnectPacketLogin::with_reason(reason)
}
