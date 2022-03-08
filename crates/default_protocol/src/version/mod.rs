use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketHandler, PacketHandlerResult};
use falcon_core::network::*;
use falcon_core::player::MinecraftPlayer;
use falcon_core::server::Difficulty;
use falcon_core::world::chunks::Chunk;

use crate::error::Result;

use crate::implement_packet_handler_enum;

// pub mod v1_8_9;
// pub mod v1_12_2;
// pub mod v1_13;
// pub mod v1_13_2;
// pub mod v1_14;

// pub mod status;

#[derive(PacketDecode)]
pub struct HandshakePacket {
    #[var_int]
    version: i32,
    address: String,
    port: u16,
    #[var_int]
    next_state: i32,
}

impl PacketHandler for HandshakePacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
        match self.next_state {
            1 => connection
                .get_handler_state_mut()
                .set_connection_state(ConnectionState::Status),
            2 => connection
                .get_handler_state_mut()
                .set_connection_state(ConnectionState::Login),
            _ => connection.disconnect(String::from("Impossible next state!")),
        }
        connection
            .get_handler_state_mut()
            .set_protocol_id(self.version);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Handshake packet"
    }
}

pub enum VersionMatcher {
    Handshake(HandshakePacket),
    // Status(StatusPackets),
    // V1_8_9(v1_8_9::PacketList),
    // V1_13(v1_13::PacketList),
    // V1_13_2(v1_13_2::PacketList),
    // V1_14(v1_14::PacketList),
}

implement_packet_handler_enum!(VersionMatcher, Handshake);

impl VersionMatcher {
    pub fn from_buf(
        packet_id: i32,
        state: &PacketHandlerState,
        buffer: &mut dyn PacketBufferRead,
    ) -> Result<Option<VersionMatcher>> {
        match state.connection_state() {
            ConnectionState::Handshake => Ok(Some(VersionMatcher::Handshake(HandshakePacket::from_buf(buffer)?))),
            // ConnectionState::Status => Ok(Some(VersionMatcher::Status(StatusPackets::from_buf(packet_id, buffer)?.ok_or(Error::from("Could not read status packet"))?))),
            _ => match state.protocol_id() {
                // PROTOCOL_1_8_9 => v1_8_9::PacketList::from_buf(packet_id, state, buffer).map(|l| l.map(VersionMatcher::V1_8_9)),
                // PROTOCOL_1_13 => v1_13::PacketList::from_buf(packet_id, state, buffer).map(|l| l.map(VersionMatcher::V1_13)),
                // PROTOCOL_1_13_2 | PROTOCOL_1_13_1 => v1_13_2::PacketList::from_buf(packet_id, state, buffer).map(|l| l.map(VersionMatcher::V1_13_2)),
                // PROTOCOL_1_14 => v1_14::PacketList::from_buf(packet_id, state, buffer).map(|l| l.map(VersionMatcher::V1_14)),
                _ => Ok(None),
            }
        }
    }
}

pub struct ProtocolSend;

impl ProtocolSend {
    pub fn join_game(player: &mut dyn MinecraftPlayer, difficulty: Difficulty, max_players: u8, level_type: String, view_distance: i32, reduced_debug: bool) -> Result<()> {
        if let Some(protocol) = ProtocolSend::get_protocol_version(player.get_protocol_version()) {
            protocol.join_game(player, difficulty, max_players, level_type, view_distance, reduced_debug)?;
        }
        Ok(())
    }

    pub fn player_abilities(player: &mut dyn MinecraftPlayer, flying_speed: f32, fov_modifier: f32) -> Result<()> {
        if let Some(protocol) = ProtocolSend::get_protocol_version(player.get_protocol_version()) {
            protocol.player_abilities(player, flying_speed, fov_modifier)?;
        }
        Ok(())
    }

    pub fn unload_chunk(player: &mut dyn MinecraftPlayer, chunk_x: i32, chunk_z: i32) -> Result<()> {
        if let Some(protocol) = ProtocolSend::get_protocol_version(player.get_protocol_version()) {
            protocol.unload_chunk(player, chunk_x, chunk_z)?;
        }
        Ok(())
    }

    pub fn send_chunk(player: &mut dyn MinecraftPlayer, chunk: &Chunk) -> Result<()> {
        if let Some(protocol) = ProtocolSend::get_protocol_version(player.get_protocol_version()) {
            protocol.send_chunk(player, chunk)?;
        }
        Ok(())
    }

    pub fn send_air_chunk(player: &mut dyn MinecraftPlayer, chunk_x: i32, chunk_z: i32) -> Result<()> {
        if let Some(protocol) = ProtocolSend::get_protocol_version(player.get_protocol_version()) {
            protocol.send_air_chunk(player, chunk_x, chunk_z)?;
        }
        Ok(())
    }

    pub fn player_position_and_look(player: &mut dyn MinecraftPlayer, flags: u8, teleport_id: i32) -> Result<()> {
        if let Some(protocol) = ProtocolSend::get_protocol_version(player.get_protocol_version()) {
            protocol.player_position_and_look(player, flags, teleport_id)?;
        }
        Ok(())
    }

    pub fn keep_alive(player: &mut dyn MinecraftPlayer, elapsed: u64) -> Result<()> {
        if let Some(protocol) = ProtocolSend::get_protocol_version(player.get_protocol_version()) {
            protocol.keep_alive(player, elapsed)?;
        }
        Ok(())
    }

    pub fn get_protocol_version<'a>(version: i32) -> Option<&'a dyn ProtocolVersioned> {
        match version {
            //PROTOCOL_1_13 => Some(&v1_13::PacketSend),
            //PROTOCOL_1_13_2 | PROTOCOL_1_13_1 => Some(&v1_13_2::PacketSend),
            //PROTOCOL_1_14 => Some(&v1_14::PacketSend),
            _ => None,
        }
    }
}

pub trait ProtocolVersioned {
    fn join_game(
        &self,
        player: &mut dyn MinecraftPlayer,
        difficulty: Difficulty,
        max_players: u8,
        level_type: String,
        view_distance: i32,
        reduced_debug: bool,
    ) -> Result<()>;

    fn player_abilities(
        &self,
        player: &mut dyn MinecraftPlayer,
        flying_speed: f32,
        fov_modifier: f32,
    ) -> Result<()>;

    fn unload_chunk(&self, player: &mut dyn MinecraftPlayer, chunk_x: i32, chunk_z: i32) -> Result<()>;

    fn send_chunk(&self, player: &mut dyn MinecraftPlayer, chunk: &Chunk) -> Result<()>;

    fn send_air_chunk(
        &self,
        player: &mut dyn MinecraftPlayer,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Result<()>;

    fn player_position_and_look(
        &self,
        player: &mut dyn MinecraftPlayer,
        flags: u8,
        teleport_id: i32,
    ) -> Result<()>;

    fn keep_alive(&self, player: &mut dyn MinecraftPlayer, elapsed: u64) -> Result<()>;
}
