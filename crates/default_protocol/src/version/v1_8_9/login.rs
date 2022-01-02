use crate::errors::*;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketEncode, PacketHandler};
use uuid::Uuid;
use falcon_core::network::ConnectionState;
use falcon_core::server::MinecraftServer;
use crate::implement_packet_handler_enum;

pub enum LoginPackets {
    LoginStart(LoginStartPacket),
}

implement_packet_handler_enum!(LoginPackets, LoginStart);

impl LoginPackets {
    pub fn from_buf(packet_id: i32, buffer: &mut dyn PacketBufferRead) -> Result<Option<LoginPackets>> {
        match packet_id {
            0x00 => Ok(Some(LoginPackets::LoginStart(LoginStartPacket::from_buf(
                buffer,
            )?))),
            _ => Ok(None),
        }
    }
}

#[derive(PacketDecode)]
pub struct LoginStartPacket {
    name: String,
}

impl PacketHandler for LoginStartPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) {
        debug!(player_name = %self.name);
        let player_uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, self.name.as_bytes());
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
        if let Err(error) = connection.get_server_link_mut().send(server_task) {
            error!("Could not join player to server due to {}!", error);
        }
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
