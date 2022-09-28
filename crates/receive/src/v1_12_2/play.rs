#[falcon_receive_derive::falcon_receive]
mod inner {
    use falcon_logic::{FalconConnection, connection::handler::PacketHandler};
    use falcon_packet_core::PacketRead;
    use mc_chat::{ChatComponent, ComponentStyle};

    #[derive(PacketRead)]
    #[falcon_packet(versions = {
        340 = 0x0B;
        393, 401, 404 = 0x0E;
        477, 480, 485, 490, 498, 573, 575, 578 = 0x0F;
        735, 736 = 0x10;
    })]
    pub struct KeepAlivePacket {
        id: i64,
    }

    impl PacketHandler for KeepAlivePacket {
        fn handle_packet(self, connection: &mut FalconConnection) {
            if connection.handler_state().last_keep_alive() != self.id as u64 {
                let version = connection.handler_state().protocol_id();
                connection.disconnect(ChatComponent::from_text("Received invalid Keep Alive id!", ComponentStyle::with_version(version.unsigned_abs())));
            } else {
                connection.reset_keep_alive();
            }
        }

        fn get_name(&self) -> &'static str {
            "Keep alive (1.12.2)"
        }
    }
}
