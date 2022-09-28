use crate::FalconConnection;

// This trait defines the packet logic when a packet gets received.
pub trait PacketHandler {
    /// Executes packet logic.
    fn handle_packet(self, connection: &mut FalconConnection);

    /// Human-readable identifier of the packet type
    fn get_name(&self) -> &'static str;
}
