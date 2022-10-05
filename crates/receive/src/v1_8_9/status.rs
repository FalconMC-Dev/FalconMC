#[falcon_receive_derive::falcon_receive]
mod inner {
    use falcon_logic::{FalconConnection, connection::handler::PacketHandler};
    use falcon_packet_core::PacketRead;
    use tracing::trace;
    use falcon_core::network::ConnectionState;

    #[derive(PacketRead)]
    #[falcon_packet(versions = { -1 = 0x00 })]
    pub struct StatusRequestPacket {}

    #[derive(PacketRead)]
    #[falcon_packet(versions = { -1 = 0x01 })]
    pub struct StatusPingPacket {
        payload: i64,
    }

    impl PacketHandler for StatusRequestPacket {
        fn handle_packet(self, connection: &mut FalconConnection) {
            trace!("Status requested");
            let version = connection.handler_state().protocol_id();
            let wrapper = connection.wrapper();
            connection.server().request_status(version, wrapper);
        }

        fn get_name(&self) -> &'static str {
            "Status request (1.8.9)"
        }
    }

    impl PacketHandler for StatusPingPacket {
        fn handle_packet(self, connection: &mut FalconConnection) {
            trace!("Sent status pong");
            connection.send_packet(self.payload, falcon_send::write_status_pong);
            connection
                .handler_state_mut()
                .set_connection_state(ConnectionState::Disconnected);
        }

        fn get_name(&self) -> &'static str {
            "Ping Request (1.8.9)"
        }
    }
}
