pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use mc_chat::{ChatColor, ChatComponent, ComponentStyle};
    use falcon_core::network::connection::ClientConnection;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_core::server::config::FalconConfig;
    use falcon_logic::connection::disconnect;
    use falcon_logic::ServerLogic;

    #[derive(PacketDecode)]
    #[falcon_packet(47, 393, 401, 404, 477, 480, 485, 490, 498, 573, 575, 578 = 0x00)]
    pub struct LoginStartPacket {
        name: String,
    }

    impl PacketHandler for LoginStartPacket {
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
            let version = connection.handler_state().protocol_id();
            if FalconConfig::global().excluded_versions().contains(&version.unsigned_abs()) {
                disconnect(connection, ChatComponent::from_text(
                    "Disabled version",
                    ComponentStyle::with_version(version.unsigned_abs()).color_if_absent(ChatColor::Red)
                ));
            } else {
                let wrapper = connection.wrapper();
                connection.server()
                    .player_login(self.name, version, wrapper);
            }
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Login Start (1.8.9)"
        }
    }
}