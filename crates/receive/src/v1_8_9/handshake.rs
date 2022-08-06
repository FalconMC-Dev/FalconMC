use falcon_logic::FalconConnection;
use mc_chat::{ChatComponent, ComponentStyle};
use falcon_core::network::connection::ConnectionLogic;
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

impl PacketHandler<FalconConnection> for HandshakePacket {
    fn handle_packet(self, connection: &mut FalconConnection) -> TaskScheduleResult {
        match self.next_state {
            1 => connection
                .handler_state_mut()
                .set_connection_state(ConnectionState::Status),
            2 => connection
                .handler_state_mut()
                .set_connection_state(ConnectionState::Login),
            _ => connection.disconnect(ChatComponent::from_text("Impossible next state!", ComponentStyle::with_version(self.version.unsigned_abs()))),
        }
        connection
            .handler_state_mut()
            .set_protocol_id(self.version);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Handshake (1.8.9)"
    }
}

