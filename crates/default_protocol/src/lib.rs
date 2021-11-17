#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate falcon_protocol;
#[macro_use]
extern crate log;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_protocol::errors::Result;
use falcon_protocol::ProtocolPlugin;

use crate::version::{DispatchPacketHandler, VersionMatcher};

pub mod errors;
pub mod version;

#[derive(Debug, Default)]
pub struct DefaultProtocol;

impl ProtocolPlugin for DefaultProtocol {
    fn name(&self) -> &'static str {
        "Default Protocol"
    }

    fn on_protocol_load(&self) {
        log4rs::init_file(falcon_core::LOG_CONFIG, Default::default()).unwrap();
    }

    fn process_packet(
        &self,
        packet_id: i32,
        buffer: &mut dyn PacketBufferRead,
        connection: &mut dyn MinecraftConnection,
    ) -> Result<Option<()>> {
        let handler_state = connection.get_handler_state();
        trace!(
            "Packet_id: {}, state: {:?}",
            packet_id,
            handler_state.get_connection_state()
        );

        VersionMatcher::from(packet_id, handler_state, buffer)
            .map(|option| {
                option.map(|packet| {
                    debug!(
                        "RECV: [{:?}: {}] {}",
                        connection.get_handler_state().get_connection_state(),
                        packet_id,
                        packet.get_name()
                    );
                    packet.handle_packet(connection)
                })
            })
            .map_err(|error| error.into())
    }
}

declare_plugin!(DefaultProtocol, Default::default);
