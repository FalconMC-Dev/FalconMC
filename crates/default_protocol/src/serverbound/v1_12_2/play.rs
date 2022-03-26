pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use mc_chat::{ChatComponent, ComponentStyle};
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};

    #[derive(PacketDecode)]
    #[falcon_packet(340 = 0x0B; 393, 401, 404 = 0x0E)]
    pub struct KeepAlivePacket {
        id: i64,
    }

    impl PacketHandler for KeepAlivePacket {
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> TaskScheduleResult {
            if connection.handler_state().last_keep_alive() != self.id as u64 {
                let version = connection.handler_state().protocol_id();
                connection.disconnect(ChatComponent::from_text("Received invalid Keep Alive id!", ComponentStyle::with_version(version.unsigned_abs())));
            } else {
                connection.reset_keep_alive();
            }
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Keep alive (1.12.2)"
        }
    }
}