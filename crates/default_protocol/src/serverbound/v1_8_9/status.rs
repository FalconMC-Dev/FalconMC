pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::ConnectionState;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use crate::clientbound::send_status_pong;

    #[derive(PacketDecode)]
    #[falcon_packet(-1 = 0x00)]
    pub struct StatusRequestPacket;

    impl PacketHandler for StatusRequestPacket {
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> TaskScheduleResult {
            debug!("Incoming status!");
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Status request (1.8.9)"
        }
    }

    #[derive(PacketDecode)]
    #[falcon_packet(-1 = 0x01)]
    pub struct StatusPingPacket {
        payload: i64,
    }

    impl PacketHandler for StatusPingPacket {
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> TaskScheduleResult {
            send_status_pong(self.payload, connection);
            connection
                .handler_state_mut()
                .set_connection_state(ConnectionState::Disconnected);
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Ping Request (1.8.9)"
        }
    }
}