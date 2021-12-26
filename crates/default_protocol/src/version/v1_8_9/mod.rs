use crate::errors::*;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::{ConnectionState, PacketHandlerState};
use crate::implement_packet_handler_enum;

use crate::version::v1_8_9::login::LoginPackets;

pub mod login;

pub enum PacketList {
    Login(LoginPackets)
}

implement_packet_handler_enum!(PacketList, Login);

impl PacketList {
    pub fn from_buf(packet_id: i32, state: &PacketHandlerState, buffer: &mut dyn PacketBufferRead) -> Result<Option<PacketList>> {
        match state.get_connection_state() {
            ConnectionState::Login => {
                LoginPackets::from_buf(packet_id, buffer).map(|l| l.map(|p| PacketList::Login(p)))
            }
            _ => Ok(None)
        }
    }
}
