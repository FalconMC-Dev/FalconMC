use enum_dispatch::enum_dispatch;

use falcon_core::errors::*;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::{ConnectionState, PacketHandlerState};

use crate::version::v1_8_9::login::LoginPackets;

pub mod login;

#[enum_dispatch(DispatchPacketHandler)]
pub enum PacketList {
    Login(LoginPackets),
}

impl PacketList {
    pub fn from(
        packet_id: i32,
        state: &PacketHandlerState,
        buffer: &mut dyn PacketBufferRead,
    ) -> Result<Option<PacketList>> {
        match state.get_connection_state() {
            ConnectionState::Login => {
                LoginPackets::from(packet_id, buffer).map(|l| l.map(|p| PacketList::Login(p)))
            }
            _ => Ok(None),
        }
    }
}
