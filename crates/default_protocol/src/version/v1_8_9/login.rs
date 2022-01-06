use uuid::Uuid;

use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::ConnectionState;
use falcon_core::network::packet::{PacketEncode, PacketDecode, PacketHandler, PacketHandlerResult, PacketHandlerError};
use falcon_core::server::MinecraftServer;

#[derive(PacketDecode)]
pub struct LoginStartPacket {
    name: String,
}

impl PacketHandler for LoginStartPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
        debug!(player_name = %self.name);
        let player_uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, self.name.as_bytes());
        // TODO: more idiomatic code pls
        connection.send_packet(
            2,
            &LoginSuccessPacket {
                uuid: player_uuid.to_hyphenated_ref().to_string(),
                username: self.name.clone(),
            },
        );
        connection.get_handler_state_mut().set_connection_state(ConnectionState::Play);
        connection.get_handler_state_mut().set_player_uuid(player_uuid);
        let server_task = {
            let name = self.name;
            let version = connection.get_handler_state().protocol_id();
            let channel = connection.get_connection_link();
            Box::new(move |server: &mut dyn MinecraftServer| {
                server.player_join(name, player_uuid, version, channel);
            })
        };
        connection.get_server_link_mut().send(server_task).map_err(|_| PacketHandlerError::ServerThreadSendError)
    }

    fn get_name(&self) -> &'static str {
        "Login Start (1.8.9)"
    }
}

#[derive(PacketEncode)]
pub struct LoginSuccessPacket {
    uuid: String,
    #[max_length(16)]
    username: String,
}
