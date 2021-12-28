#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_new;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::PacketEncode;

use crate::version::VersionMatcher;
use errors::*;
use falcon_core::network::packet::PacketHandler;

pub use version::ProtocolSend;

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
        trace!(
            "Packet ID: {:#04X}, state: {:?}",
            packet_id,
            handler_state.connection_state()
        );

        VersionMatcher::from_buf(packet_id, handler_state, buffer).map(|option| {
            option.map(|packet| {
                trace!(
                    "RECV: [{:?}: {:#04X}] {}",
                    connection.get_handler_state().connection_state(),
                    packet_id,
                    packet.get_name()
                );
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
