pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_core::server::ServerActor;

    #[derive(PacketDecode)]
    #[falcon_packet(47 = 0x00)]
    pub struct LoginStartPacket {
        name: String,
    }

    impl PacketHandler for LoginStartPacket {
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> TaskScheduleResult {
            let version = connection.handler_state().protocol_id();
            let wrapper = connection.wrapper();
            connection.server()
                .player_login(self.name, version, wrapper);
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Login Start (1.8.9)"
        }
    }
}