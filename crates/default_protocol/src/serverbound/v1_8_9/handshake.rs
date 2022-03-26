pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use mc_chat::{ChatComponent, ComponentStyle};
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::ConnectionState;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};

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
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> TaskScheduleResult {
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
}
