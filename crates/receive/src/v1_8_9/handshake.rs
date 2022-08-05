use falcon_logic::FalconConnection;
use mc_chat::{ChatComponent, ComponentStyle};
use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic};
use falcon_core::network::ConnectionState;
use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};

falcon_receive_derive::falcon_receive! {
    #[derive(PacketDecode)]
    #[falcon_packet(versions = { -1 = 0x00 })]
    pub struct HandshakePacket {
        #[var_int]
        version: i32,
        address: String,
        port: u16,
        #[var_int]
        next_state: i32,
    }
}

impl<D: ConnectionDriver + 'static> PacketHandler<D, FalconConnection<D>> for HandshakePacket {
    fn handle_packet(self, connection: &mut FalconConnection<D>) -> TaskScheduleResult {
        match self.next_state {
            1 => connection
                .driver_mut()
                .handler_state_mut()
                .set_connection_state(ConnectionState::Status),
            2 => connection
                .driver_mut()
                .handler_state_mut()
                .set_connection_state(ConnectionState::Login),
            _ => connection.disconnect(ChatComponent::from_text("Impossible next state!", ComponentStyle::with_version(self.version.unsigned_abs()))),
        }
        connection
            .driver_mut()
            .handler_state_mut()
            .set_protocol_id(self.version);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Handshake (1.8.9)"
    }
}

