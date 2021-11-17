use enum_dispatch::enum_dispatch;

use falcon_core::errors::*;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketHandler};
use falcon_core::network::{ConnectionState, PacketHandlerState};

use crate::version::v1_8_9::login::{LoginPackets, LoginStartPacket};
use crate::version::v1_8_9::PacketList;

pub mod v1_8_9;

const PROTOCOL_1_8_9: i32 = 47;

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
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) {
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
    }

    fn get_name(&self) -> &'static str {
        "Handshake packet"
    }
}

#[enum_dispatch(DispatchPacketHandler)]
pub enum VersionMatcher {
    Handshake(HandshakePacket),
    V1_8_9(v1_8_9::PacketList),
}

impl VersionMatcher {
    pub fn from(
        packet_id: i32,
        state: &PacketHandlerState,
        buffer: &mut dyn PacketBufferRead,
    ) -> Result<Option<VersionMatcher>> {
        if state.get_connection_state() == ConnectionState::Handshake {
            Ok(Some(VersionMatcher::Handshake(HandshakePacket::from_buf(
                buffer,
            )?)))
        } else {
            match state.get_protocol_id() {
                47 => v1_8_9::PacketList::from(packet_id, state, buffer)
                    .map(|l| l.map(|p| VersionMatcher::V1_8_9(p))),
                _ => Ok(None),
            }
        }
    }
}

#[enum_dispatch]
pub trait DispatchPacketHandler {
    /// Executes packet logic.
    fn handle_packet(self, connection: &mut dyn MinecraftConnection);

    /// Human-readable identifier of the packet type
    fn get_name(&self) -> &'static str;
}

impl<T: PacketHandler> DispatchPacketHandler for T {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) {
        self.handle_packet(connection);
    }

    fn get_name(&self) -> &'static str {
        self.get_name()
    }
}
