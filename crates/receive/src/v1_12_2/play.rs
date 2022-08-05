use falcon_logic::FalconConnection;
use mc_chat::{ChatComponent, ComponentStyle};
use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic};
use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};

falcon_receive_derive::falcon_receive! {
    #[derive(PacketDecode)]
    #[falcon_packet(versions = {
        340 = 0x0B;
        393, 401, 404 = 0x0E;
        477, 480, 485, 490, 498, 573, 575, 578 = 0x0F;
        735, 736 = 0x10;
    })]
    pub struct KeepAlivePacket {
        id: i64,
    }
}

impl<D: ConnectionDriver + 'static> PacketHandler<D, FalconConnection<D>> for KeepAlivePacket {
    fn handle_packet(self, connection: &mut FalconConnection<D>) -> TaskScheduleResult {
        if connection.driver().handler_state().last_keep_alive() != self.id as u64 {
            let version = connection.driver().handler_state().protocol_id();
            connection.disconnect(ChatComponent::from_text("Received invalid Keep Alive id!", ComponentStyle::with_version(version.unsigned_abs())));
        } else {
            connection.reset_keep_alive();
        }
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Keep alive (1.12.2)"
    }
}
