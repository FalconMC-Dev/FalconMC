use std::error::Error;

use crate::FalconConnection;

// This trait defines the packet logic when a packet gets received.
pub trait PacketHandler {
    /// The error that can occur when executing the packet logic
    type Error: Error + Send + Sync + 'static;

    /// Executes packet logic.
    fn handle_packet(self, connection: &mut FalconConnection) -> Result<(), Self::Error>;

    /// Human-readable identifier of the packet type
    fn get_name(&self) -> &'static str;
}
