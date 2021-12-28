//! Part of the Public API of FalconMC

use uuid::Uuid;

pub mod buffer;
pub mod connection;
pub mod packet;

pub const PROTOCOL_1_8_9: i32 = 47;
pub const PROTOCOL_1_13_2: i32 = 404;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PacketHandlerState {
    uuid: Option<Uuid>,
    protocol_id: i32,
    connection_state: ConnectionState,
}

impl PacketHandlerState {
    pub fn new(protocol_id: i32) -> PacketHandlerState {
        PacketHandlerState {
            uuid: None,
            protocol_id,
            connection_state: ConnectionState::Handshake,
        }
    }

    pub fn get_player_uuid(&self) -> Option<Uuid> {
        self.uuid
    }

    pub fn set_player_uuid(&mut self, uuid: Uuid) {
        self.uuid = Some(uuid);
    }

    pub fn get_protocol_id(&self) -> i32 {
        self.protocol_id
    }

    pub fn set_protocol_id(&mut self, protocol_id: i32) {
        self.protocol_id = protocol_id;
    }

    pub fn get_connection_state(&self) -> ConnectionState {
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
