use ignore_result::Ignore;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketEncode, PacketHandler, PacketHandlerError, PacketHandlerResult};

use serde::Serialize;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::ConnectionState;
use falcon_core::server::config::FalconConfig;
use falcon_core::server::{MinecraftServer, ServerVersion};

pub struct StatusRequestPacket;
impl PacketDecode for StatusRequestPacket {
    fn from_buf(_buf: &mut dyn PacketBufferRead) -> falcon_core::errors::Result<Self> { Ok(StatusRequestPacket) }
}

impl PacketHandler for StatusRequestPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
        let connection_link = connection.get_connection_link();
        // TODO: send status to unsupported versions too (see connection.rs for auto-disconnect when `Status` packet is unknown)
        let version = ServerVersion::new(String::from("1.13-1.17.1"), connection.get_handler_state().protocol_id());
        let server_task = {
            let description = String::from("§eFalcon server§r§b!!!");
            Box::new(|server: &mut dyn MinecraftServer| {
                let connection = connection_link;
                let player_data = PlayerData::new(FalconConfig::global().max_players(), server.online_player_count());
                let packet_data = StatusResponseData::new(version, player_data, description);
                let response = match serde_json::to_string(&packet_data) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(data = ?packet_data, error = %e, "failed to serialize status response");
                        connection.send(Box::new(|connection| connection.disconnect(String::from("Failed to send status!")))).ignore();
                        return;
                    }
                };
                connection.send(Box::new(|connection| {
                    let packet_out = StatusResponsePacket::new(response);
                    connection.send_packet(0x00, &packet_out);
                })).ignore()
            })
        };
        connection.get_server_link_mut().send(server_task).map_err(|_| PacketHandlerError::ServerThreadSendError)
    }

    fn get_name(&self) -> &'static str {
        "Status Request (1.8.9)"
    }
}

#[derive(PacketDecode)]
pub struct ServerPingPacket {
    payload: i64,
}

impl PacketHandler for ServerPingPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
        connection.send_packet(0x01, &ClientPingPacket::new(self.payload));
        connection.get_handler_state_mut().set_connection_state(ConnectionState::Disconnected);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Ping Request (1.8.9)"
    }
}

#[derive(PacketEncode, new)]
pub struct StatusResponsePacket {
    #[max_length(32767)]
    response: String,
}

#[derive(Debug, Serialize, new)]
struct StatusResponseData {
    version: ServerVersion,
    players: PlayerData,
    description: String,
}

#[derive(Debug, Serialize, new)]
struct PlayerData {
    max: i32,
    online: i32,
}

#[derive(PacketEncode, new)]
pub struct ClientPingPacket {
    payload: i64,
}