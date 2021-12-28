use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::packet::PacketDecode;

use crate::errors::*;
use crate::implement_packet_handler_enum;
use crate::version::v1_12_2::play::KeepAlivePacket;
use crate::version::v1_8_9::play::{PlayerLookPacket, PlayerPositionAndLookPacket, PlayerPositionPacket};

pub enum PlayPackets {
    KeepAlive(KeepAlivePacket),
    PlayerPosition(PlayerPositionPacket),
    PlayerPositionAndLook(PlayerPositionAndLookPacket),
    PlayerLook(PlayerLookPacket),
}

implement_packet_handler_enum!(PlayPackets, KeepAlive, PlayerPosition, PlayerPositionAndLook, PlayerLook);

impl PlayPackets {
    pub fn from_buf(packet_id: i32, buffer: &mut dyn PacketBufferRead) -> Result<Option<PlayPackets>> {
        match packet_id {
            0x0E => Ok(Some(PlayPackets::KeepAlive(KeepAlivePacket::from_buf(buffer)?))),
            0x10 => Ok(Some(PlayPackets::PlayerPosition(PlayerPositionPacket::from_buf(buffer)?))),
            0x11 => Ok(Some(PlayPackets::PlayerPositionAndLook(PlayerPositionAndLookPacket::from_buf(buffer)?))),
            0x12 => Ok(Some(PlayPackets::PlayerLook(PlayerLookPacket::from_buf(buffer)?))),
            _ => Ok(None),
        }
    }
}

