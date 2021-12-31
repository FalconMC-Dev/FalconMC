#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate tracing;

use errors::*;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::PacketEncode;
use falcon_core::network::packet::PacketHandler;
pub use version::ProtocolSend;

use crate::version::VersionMatcher;

pub mod errors;
pub mod version;
pub(crate) mod macros;

#[derive(Debug, Default)]
pub struct DefaultProtocol;

impl DefaultProtocol {
    pub fn process_packet<R: PacketBufferRead, C: MinecraftConnection>(
        packet_id: i32,
        buffer: &mut R,
        connection: &mut C,
    ) -> Result<Option<()>> {
        let handler_state = connection.get_handler_state();
        let span = info_span!("default_process_packet", packet_id = %format!("{:#04X}", packet_id), state = ?handler_state.connection_state());
        let _enter = span.enter();

        VersionMatcher::from_buf(packet_id, handler_state, buffer).map(|option| {
            option.map(|packet| {
                let packet_span = info_span!("handle_packet", name = packet.get_name());
                let _enter2 = packet_span.enter();
                packet.handle_packet(connection);
            })
        })
    }
}

#[derive(PacketEncode)]
pub struct DisconnectPacketLogin {
    #[max_length(262144)]
    reason: String,
}

impl DisconnectPacketLogin {
    pub fn with_reason(reason: String) -> Self {
        DisconnectPacketLogin {
            reason,
        }
    }
}
