use falcon_core::network::{ConnectionState, PacketHandlerState};
use falcon_core::network::buffer::PacketBufferRead;
use falcon_default_protocol_derive::PacketEnum;

use crate::errors::*;
use crate::implement_packet_handler_enum;

pub use super::v1_8_9::{LoginStartPacket, PlayerPositionPacket, PlayerPositionAndLookPacketIn, PlayerLookPacket};
pub use super::v1_12_2::KeepAlivePacket;
pub use super::v1_13::{JoinGamePacket, PlayerAbilitiesPacket, ChunkDataPacket, PlayerPositionAndLookPacketOut};

pub mod send;

pub enum PacketList {
    Login(LoginPackets),
    Play(PlayPackets),
}

implement_packet_handler_enum!(PacketList, Login, Play);

impl PacketList {
    pub fn from_buf(
        packet_id: i32,
        state: &PacketHandlerState,
        buffer: &mut dyn PacketBufferRead,
    ) -> Result<Option<PacketList>> {
        match state.connection_state() {
            ConnectionState::Login => {
                LoginPackets::from_buf(packet_id, buffer).map(|l| l.map(PacketList::Login))
            }
            ConnectionState::Play => {
                PlayPackets::from_buf(packet_id, buffer).map(|l| l.map(PacketList::Play))
            }
            _ => Ok(None),
        }
    }
}

#[derive(PacketEnum)]
pub enum LoginPackets {
    #[falcon_packet(id = 0x00)]
    LoginStart(LoginStartPacket),
}

implement_packet_handler_enum!(LoginPackets, LoginStart);

#[derive(PacketEnum)]
pub enum PlayPackets {
    #[falcon_packet(id = 0x0E)]
    KeepAlive(KeepAlivePacket),
    #[falcon_packet(id = 0x10)]
    PlayerPosition(PlayerPositionPacket),
    #[falcon_packet(id = 0x11)]
    PlayerPositionAndLook(PlayerPositionAndLookPacketIn),
    #[falcon_packet(id = 0x12)]
    PlayerLook(PlayerLookPacket),
}

implement_packet_handler_enum!(PlayPackets, KeepAlive, PlayerPosition, PlayerPositionAndLook, PlayerLook);
