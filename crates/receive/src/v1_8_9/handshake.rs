pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use mc_chat::{ChatComponent, ComponentStyle};
    use falcon_core::network::connection::ClientConnection;
    use falcon_core::network::ConnectionState;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_logic::connection::disconnect;

    #[derive(PacketDecode)]
    #[falcon_packet(-1 = 0x00)]
    pub struct HandshakePacket {
        #[var_int]
        version: i32,
        address: String,
        port: u16,
        #[var_int]
        next_state: i32,
    }

    impl PacketHandler for HandshakePacket {
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
            match self.next_state {
                1 => connection
                    .handler_state_mut()
                    .set_connection_state(ConnectionState::Status),
                2 => connection
                    .handler_state_mut()
                    .set_connection_state(ConnectionState::Login),
                _ => disconnect(connection, ChatComponent::from_text("Impossible next state!", ComponentStyle::with_version(self.version.unsigned_abs()))),
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
}
