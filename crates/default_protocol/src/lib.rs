#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;

use crate::version::{DispatchPacketHandler, VersionMatcher};
use errors::*;

pub mod errors;
pub mod version;

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
            "Packet ID: {}, state: {:?}",
            packet_id,
            handler_state.get_connection_state()
        );

        VersionMatcher::from(packet_id, handler_state, buffer).map(|option| {
            option.map(|packet| {
                trace!(
                    "RECV: [{:?}: {}] {}",
                    connection.get_handler_state().get_connection_state(),
                    packet_id,
                    packet.get_name()
                );
                packet.handle_packet(connection);
            })
        })
    }
}
