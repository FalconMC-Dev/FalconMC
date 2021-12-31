use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketHandler};
use falcon_core::server::MinecraftServer;

use crate::errors::*;
use crate::implement_packet_handler_enum;

pub enum PlayPackets {
    PlayerPosition(PlayerPositionPacket),
    PlayerPositionAndLook(PlayerPositionAndLookPacket),
    PlayerLook(PlayerLookPacket),
}

implement_packet_handler_enum!(PlayPackets, PlayerPosition, PlayerPositionAndLook, PlayerLook);

impl PlayPackets {
    pub fn from_buf(packet_id: i32, buffer: &mut dyn PacketBufferRead) -> Result<Option<PlayPackets>> {
        match packet_id {
            0x04 => Ok(Some(PlayPackets::PlayerPosition(PlayerPositionPacket::from_buf(buffer)?))),
            0x05 => Ok(Some(PlayPackets::PlayerLook(PlayerLookPacket::from_buf(buffer)?))),
            0x06 => Ok(Some(PlayPackets::PlayerPositionAndLook(PlayerPositionAndLookPacket::from_buf(buffer)?))),
            _ => Ok(None),
        }
    }
}

#[derive(PacketDecode)]
pub struct PlayerPositionPacket {
    x: f64,
    y: f64,
    z: f64,
    on_ground: bool,
}

impl PacketHandler for PlayerPositionPacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) {
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
        if let Err(error) = connection.get_server_link_mut().send(server_task) {
            error!("Could not send server task when received player position update! {}", error);
        }
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
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) {
        let server_task = {
            let uuid = connection.get_handler_state().player_uuid().expect("Should be impossible to receive this packet without a uuid");
            Box::new(move |server: &mut dyn MinecraftServer| {
                let packet = self;
                server.player_position_and_look(uuid, packet.x, packet.y, packet.z, packet.yaw, packet.pitch, packet.on_ground);
            })
        };
        if let Err(error) = connection.get_server_link_mut().send(server_task) {
            error!("Could not send server task when received player position and look update! {}", error);
        }
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
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) {
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
        if let Err(error) = connection.get_server_link_mut().send(server_task) {
            error!("Could not send server task when received player position update! {}", error);
        }
    }

    fn get_name(&self) -> &'static str {
        "Player Look (1.8.9)"
    }
}