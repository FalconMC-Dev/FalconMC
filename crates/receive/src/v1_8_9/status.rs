pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::connection::ClientConnection;
    use falcon_core::network::ConnectionState;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_logic::ServerLogic;
    use falcon_send::send_status_pong;

    #[derive(PacketDecode)]
    #[falcon_packet(-1 = 0x00)]
    pub struct StatusRequestPacket;

    impl PacketHandler for StatusRequestPacket {
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
            let version = connection.handler_state().protocol_id();
            let wrapper = connection.wrapper();
            connection.server()
                .request_status(version, wrapper);
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
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
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