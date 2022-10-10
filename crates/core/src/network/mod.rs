//! Part of the Public API of FalconMC

use std::fmt::{Display, Formatter};

use uuid::Uuid;

pub mod util;

pub const UNKNOWN_PROTOCOL: i32 = -1;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PacketHandlerState {
    pub uuid: Option<Uuid>,
    pub last_keep_alive: u64,
    pub protocol_id: i32,
    pub connection_state: ConnectionState,
}

impl Display for PacketHandlerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}|{}", self.connection_state, self.protocol_id) }
}

impl PacketHandlerState {
    pub fn new(protocol_id: i32) -> PacketHandlerState {
        PacketHandlerState {
            uuid: None,
            last_keep_alive: 0,
            protocol_id,
            connection_state: ConnectionState::Handshake,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
    Disconnected,
}
