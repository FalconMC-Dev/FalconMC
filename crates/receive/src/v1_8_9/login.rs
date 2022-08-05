use mc_chat::{ChatColor, ChatComponent, ComponentStyle};
use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic};
use falcon_logic::connection::FalconConnection;
use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
use falcon_core::server::config::FalconConfig;

falcon_receive_derive::falcon_receive! {
    #[derive(PacketDecode)]
    #[falcon_packet(versions = {
        47, 393, 401, 404, 477, 480, 485, 490, 498, 573, 575, 578, 735, 736 = 0x00
    })]
    pub struct LoginStartPacket {
        name: String,
    }
}

impl<D: ConnectionDriver + 'static> PacketHandler<D, FalconConnection<D>> for LoginStartPacket {
    fn handle_packet(self, connection: &mut FalconConnection<D>) -> TaskScheduleResult {
        let version = connection.driver().handler_state().protocol_id();
        if FalconConfig::global().excluded_versions().contains(&version.unsigned_abs()) {
            connection.disconnect(ChatComponent::from_text(
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
