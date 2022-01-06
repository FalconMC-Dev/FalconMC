use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketHandler, PacketHandlerError, PacketHandlerResult};
use falcon_core::server::MinecraftServer;

#[derive(PacketDecode)]
pub struct PlayerPositionPacket {
    x: f64,
    y: f64,
    z: f64,
    on_ground: bool,
}

impl PacketHandler for PlayerPositionPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
        let server_task = {
            let uuid = connection.get_handler_state().player_uuid().expect("Should be impossible to receive this packet without a uuid");
            Box::new(move |server: &mut dyn MinecraftServer| {
                let packet = self;
                let player = server.get_player(uuid);
                match player {
                    Some(player) => {
                        let yaw = player.get_look_angles().get_yaw();
                        let pitch = player.get_look_angles().get_pitch();
                        server.player_position_and_look(uuid, packet.x, packet.y, packet.z, yaw, pitch, packet.on_ground);
                    }
                    None => error!("We received a packet from a player that is not existing in the world!"),
                }
            })
        };
        connection.get_server_link_mut().send(server_task).map_err(|_| PacketHandlerError::ServerThreadSendError)
    }

    fn get_name(&self) -> &'static str {
        "Player Position (1.8.9)"
    }
}

#[derive(PacketDecode)]
pub struct PlayerPositionAndLookPacket {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}

impl PacketHandler for PlayerPositionAndLookPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
        let server_task = {
            let uuid = connection.get_handler_state().player_uuid().expect("Should be impossible to receive this packet without a uuid");
            Box::new(move |server: &mut dyn MinecraftServer| {
                let packet = self;
                server.player_position_and_look(uuid, packet.x, packet.y, packet.z, packet.yaw, packet.pitch, packet.on_ground);
            })
        };
        connection.get_server_link_mut().send(server_task).map_err(|_| PacketHandlerError::ServerThreadSendError)
    }

    fn get_name(&self) -> &'static str {
        "Player Position And Look (1.8.9)"
    }
}

#[derive(PacketDecode)]
pub struct PlayerLookPacket {
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}

impl PacketHandler for PlayerLookPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
        let server_task = {
            let uuid = connection.get_handler_state().player_uuid().expect("Should be impossible to receive this packet without a uuid");
            Box::new(move |server: &mut dyn MinecraftServer| {
                let packet = self;
                let player = server.get_player(uuid);
                match player {
                    Some(player) => {
                        let x = player.get_position().get_x();
                        let y = player.get_position().get_y();
                        let z = player.get_position().get_z();
                        server.player_position_and_look(uuid, x, y, z, packet.yaw, packet.pitch, packet.on_ground);
                    }
                    None => error!("We received a packet from a player that is not to be found on the server!"),
                }
            })
        };
        connection.get_server_link_mut().send(server_task).map_err(|_| PacketHandlerError::ServerThreadSendError)
    }

    fn get_name(&self) -> &'static str {
        "Player Look (1.8.9)"
    }
}