#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate falcon_protocol;
#[macro_use]
extern crate log;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::ConnectionState;
use falcon_protocol::ProtocolPlugin;

use falcon_protocol::errors::*;

pub mod errors;

const PROTOCOL_1_8_9: i32 = 47;

#[derive(Debug, Default)]
pub struct DefaultProtocol;

impl ProtocolPlugin for DefaultProtocol {
    fn name(&self) -> &'static str {
        "Default Protocol"
    }

    fn on_protocol_load(&self) {
        log4rs::init_file(falcon_core::LOG_CONFIG, Default::default()).unwrap();
    }

    fn process_packet(&self, packet_id: i32, buffer: &mut dyn PacketBufferRead, connection: &mut dyn MinecraftConnection) -> Option<Result<()>> {
        let conn_state = connection.get_handler_state().get_connection_state();
        trace!("Packet_id: {}, state: {:?}", packet_id, conn_state);
        None
    }
}

declare_plugin!(DefaultProtocol, Default::default);