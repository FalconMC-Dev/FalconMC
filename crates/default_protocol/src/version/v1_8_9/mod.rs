use falcon_core::network::{ConnectionState, PacketHandlerState};
use falcon_core::network::buffer::PacketBufferRead;
use falcon_default_protocol_derive::PacketEnum;

use crate::errors::*;
use crate::implement_packet_handler_enum;

pub use self::status::{StatusRequestPacket, StatusResponsePacket, ClientPingPacket, ServerPingPacket};
pub use self::login::LoginStartPacket;
pub use self::play::{PlayerLookPacket, PlayerPositionAndLookPacket as PlayerPositionAndLookPacketIn, PlayerPositionPacket};

mod login;
mod play;
mod status;
mod util;

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
    #[falcon_packet(id = 0x04)]
    PlayerPosition(PlayerPositionPacket),
    #[falcon_packet(id = 0x05)]
    PlayerPositionAndLook(PlayerPositionAndLookPacketIn),
    #[falcon_packet(id = 0x06)]
    PlayerLook(PlayerLookPacket),
}
implement_packet_handler_enum!(PlayPackets, PlayerPosition, PlayerPositionAndLook, PlayerLook);
