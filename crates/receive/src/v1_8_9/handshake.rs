#[falcon_receive_derive::falcon_receive]
mod inner {
    use std::convert::Infallible;

    use falcon_logic::{FalconConnection, connection::handler::PacketHandler};
    use falcon_packet_core::PacketRead;
    use mc_chat::{ChatComponent, ComponentStyle};
    use falcon_core::network::ConnectionState;

    #[derive(PacketRead)]
    #[falcon_packet(versions = { -1 = 0x00 })]
    pub struct HandshakePacket {
        #[falcon(var32)]
        version: i32,
        #[falcon(string)]
        address: String,
        port: u16,
        #[falcon(var32)]
        next_state: i32,
    }

    impl PacketHandler for HandshakePacket {
        type Error = Infallible;

        fn handle_packet(self, connection: &mut FalconConnection) -> Result<(), Infallible> {
            match self.next_state {
                1 => connection
                    .handler_state_mut()
                    .set_connection_state(ConnectionState::Status),
                2 => connection
                    .handler_state_mut()
                    .set_connection_state(ConnectionState::Login),
                _ => {
                    connection.disconnect(ChatComponent::from_text("Impossible next state!", ComponentStyle::with_version(self.version.unsigned_abs())));
                }
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
