//! Part of the Public API of FalconMC

use std::fmt::{Display, Formatter};

use uuid::Uuid;

pub mod util;

pub const UNKNOWN_PROTOCOL: i32 = -1;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PacketHandlerState {
    uuid: Option<Uuid>,
    last_keep_alive: u64,
    protocol_id: i32,
    connection_state: ConnectionState,
}

impl Display for PacketHandlerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}|{}", self.connection_state, self.protocol_id)
    }
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

    pub fn player_uuid(&self) -> Option<Uuid> {
        self.uuid
    }

    pub fn set_player_uuid(&mut self, uuid: Uuid) {
        self.uuid = Some(uuid);
    }

    pub fn last_keep_alive(&self) -> u64 {
        self.last_keep_alive
    }

    pub fn set_last_keep_alive(&mut self, last_keep_alive: u64) {
        self.last_keep_alive = last_keep_alive;
    }

    pub fn protocol_id(&self) -> i32 {
        self.protocol_id
    }

    pub fn set_protocol_id(&mut self, protocol_id: i32) {
        self.protocol_id = protocol_id;
    }

    pub fn connection_state(&self) -> ConnectionState {
        self.connection_state
    }

    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
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
