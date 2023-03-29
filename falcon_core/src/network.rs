//! A collection of base structs used when dealing with
//! minecraft networking.

use std::fmt::{Display, Formatter};

use uuid::Uuid;

pub const UNKNOWN_PROTOCOL: i32 = -1;

/// Main context for a network connection.
///
/// This context stores:
/// - The protocol phase of the connection
/// - The protocol version of the connection
/// - The timestamp from the last keepalive check
/// - A uuid (optional) of the player identified with the connection.
///
/// # Note
/// This could be moved to a higher crate such as the logic core crates.
///
/// (For example clients might want to have a plain `Uuid` instead of an
/// `Option<Uuid>`)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NetworkState {
    pub uuid: Option<Uuid>,
    pub keepalive: u64,
    pub protocol: i32,
    pub phase: Phase,
}

impl Display for NetworkState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}|{}", self.phase, self.protocol) }
}

impl NetworkState {
    pub fn new(protocol: i32) -> NetworkState {
        NetworkState {
            uuid: None,
            keepalive: 0,
            protocol,
            phase: Phase::Handshake,
        }
    }
}

/// The minecraft protocol specifies 4 contexts where
/// packets are interpreted in. The [`Phase::Disconnected`]
/// context is added to easily distinguish between open
/// and closed connections.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Phase {
    Handshake,
    Status,
    Login,
    Play,
    Disconnected,
}
