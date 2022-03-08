use falcon_core::player::MinecraftPlayer;
use falcon_core::server::Difficulty;
use falcon_core::world::blocks::Blocks;
use falcon_core::world::chunks::Chunk;

use crate::errors::*;
use crate::version::ProtocolVersioned;
use super::util::easy_send;

use super::play::{
    ChunkDataPacket, JoinGamePacket, PlayerAbilitiesPacket, PlayerPositionAndLookPacket,
};
use crate::version::v1_12_2::play::KeepAlivePacket;
use crate::version::v1_13::play::UnloadChunkPacket;

pub struct PacketSend;

impl ProtocolVersioned for PacketSend {
    fn join_game(&self, player: &mut dyn MinecraftPlayer, difficulty: Difficulty, max_players: u8, level_type: String, _view_distance: i32, reduced_debug: bool) -> Result<()> {
        let packet = JoinGamePacket::new(player.get_entity_id(), player.get_game_mode(), player.get_dimension(), difficulty, max_players, level_type, reduced_debug);
        easy_send(player.get_client_connection(), 0x25, packet)
            .map_err(|_| Error::from("Could not send join game packet"))
    }

    fn player_abilities(&self, player: &mut dyn MinecraftPlayer, flying_speed: f32, fov_modifier: f32) -> Result<()> {
        let packet = PlayerAbilitiesPacket::new(player.get_ability_flags(), flying_speed, fov_modifier);
        easy_send(player.get_client_connection(), 0x2E, packet)
            .map_err(|_| Error::from("Could not send player abilities packet"))
    }

    fn unload_chunk(&self, player: &mut dyn MinecraftPlayer, chunk_x: i32, chunk_z: i32) -> Result<()> {
        let packet = UnloadChunkPacket::new(chunk_x, chunk_z);
        easy_send(player.get_client_connection(), 0x1F, packet)
            .map_err(|_| Error::from("Could not send unload chunk packet"))
    }

    fn send_chunk(&self, player: &mut dyn MinecraftPlayer, chunk: &Chunk) -> Result<()> {
        let packet = ChunkDataPacket::from_chunk(chunk, Blocks::get_global_id_1519);
        easy_send(player.get_client_connection(), 0x22, packet)
            .map_err(|_| Error::from("Could not send chunk data"))
    }

    fn send_air_chunk(&self, player: &mut dyn MinecraftPlayer, chunk_x: i32, chunk_z: i32) -> Result<()> {
        player.get_client_connection().send(Box::new(move |conn| {
            let packet = ChunkDataPacket::empty(chunk_x, chunk_z);
            conn.send_packet(0x22, &packet);
        })).map_err(|_| Error::from("Could not send air chunk data"))
    }

    fn player_position_and_look(&self, player: &mut dyn MinecraftPlayer, flags: u8, teleport_id: i32) -> Result<()> {
        let pos = *player.get_position();
        let look = *player.get_look_angles();
        let packet = PlayerPositionAndLookPacket::new(pos.get_x(), pos.get_y(), pos.get_z(), look.get_yaw(), look.get_pitch(), flags, teleport_id);
        easy_send(player.get_client_connection(), 0x32, packet)
            .map_err(|_| Error::from("Could not send player position and look packet"))
    }

    fn keep_alive(&self, player: &mut dyn MinecraftPlayer, elapsed: u64) -> Result<()> {
        player.get_client_connection().send(Box::new(move |conn| {
            let packet_out = KeepAlivePacket::new(elapsed as i64);
            conn.get_handler_state_mut().set_last_keep_alive(elapsed);
            conn.send_packet(0x21, &packet_out);
        })).map_err(|_| Error::from("Could not send Keep Alive packet"))
    }
}