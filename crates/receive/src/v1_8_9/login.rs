#[falcon_receive_derive::falcon_receive]
mod inner {
    use std::convert::Infallible;

    use falcon_packet_core::PacketRead;
    use mc_chat::{ChatColor, ChatComponent, ComponentStyle};
    use falcon_logic::connection::{FalconConnection, handler::PacketHandler};
    use falcon_core::server::config::FalconConfig;

    #[derive(PacketRead)]
    #[falcon_packet(versions = {
        47, 393, 401, 404, 477, 480, 485, 490, 498, 573, 575, 578, 735, 736 = 0x00
    })]
    pub struct LoginStartPacket {
        #[falcon(string = 16)]
        name: String,
    }

    impl PacketHandler for LoginStartPacket {
        type Error = Infallible;

        fn handle_packet(self, connection: &mut FalconConnection) -> Result<(), Self::Error> {
            let version = connection.state().protocol_id;

            if !FalconConfig::ALLOWED_VERSIONS.contains(&version.unsigned_abs()) {
                 connection.disconnect(ChatComponent::from_text(
                     "Incompatible version",
                     ComponentStyle::with_version(version.unsigned_abs()).color_if_absent(ChatColor::Red)
                ));
            } 
            if FalconConfig::global().versions.excluded.contains(&version.unsigned_abs()) {
                connection.disconnect(ChatComponent::from_text(
                    "Disabled version",
                    ComponentStyle::with_version(version.unsigned_abs()).color_if_absent(ChatColor::Red)
                ));
            } else {
                let wrapper = connection.wrapper();
                connection.server().player_login(self.name, version, wrapper);
            }
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Login Start (1.8.9)"
        }
    }
}
