use falcon_core::network::{ConnectionState, PacketHandlerState};
use falcon_core::network::buffer::PacketBufferRead;
use falcon_default_protocol_derive::PacketEnum;

use crate::errors::*;
use crate::implement_packet_handler_enum;

pub use super::v1_12_2::KeepAlivePacket;
pub use super::v1_8_9::{StatusRequestPacket, ServerPingPacket, LoginStartPacket, PlayerLookPacket, PlayerPositionAndLookPacketIn, PlayerPositionPacket};
pub use play::{JoinGamePacket, PlayerAbilitiesPacket, ChunkDataPacket, PlayerPositionAndLookPacket as PlayerPositionAndLookPacketOut};
pub use send::PacketSend;

mod send;
mod util;
mod play;

pub enum PacketList {
    Status(StatusPackets),
    Login(LoginPackets),
    Play(PlayPackets),
}
implement_packet_handler_enum!(PacketList, Status, Login, Play);

impl PacketList {
    pub fn from_buf(packet_id: i32, state: &PacketHandlerState, buffer: &mut dyn PacketBufferRead) -> Result<Option<PacketList>> {
        match state.connection_state() {
            ConnectionState::Status => {
                StatusPackets::from_buf(packet_id, buffer).map(|l| l.map(PacketList::Status))
            }
            ConnectionState::Login => {
                LoginPackets::from_buf(packet_id, buffer).map(|l| l.map(PacketList::Login))
            }
            ConnectionState::Play => {
                PlayPackets::from_buf(packet_id, buffer).map(|l| l.map(PacketList::Play))
            }
            _ => Ok(None)
        }
    }
}

#[derive(PacketEnum)]
pub enum StatusPackets {
    #[falcon_packet(id = 0x00)]
    Request(StatusRequestPacket),
    #[falcon_packet(id = 0x01)]
    Ping(ServerPingPacket),
}
implement_packet_handler_enum!(StatusPackets, Request, Ping);

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

