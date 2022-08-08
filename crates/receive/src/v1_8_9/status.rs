use falcon_core::network::connection::ConnectionLogic;
use falcon_logic::FalconConnection;
use tracing::trace;

falcon_receive_derive::falcon_receive! {
    use falcon_core::network::ConnectionState;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};

    #[derive(PacketDecode)]
    #[falcon_packet(versions = { -1 = 0x00 })]
    pub struct StatusRequestPacket;

    #[derive(PacketDecode)]
    #[falcon_packet(versions = { -1 = 0x01 })]
    pub struct StatusPingPacket {
        payload: i64,
    }
}

impl PacketHandler<FalconConnection> for StatusRequestPacket {
    fn handle_packet(self, connection: &mut FalconConnection) -> TaskScheduleResult {
        trace!("Status requested");
        let version = connection.handler_state().protocol_id();
        let wrapper = connection.wrapper();
        connection.server().request_status(version, wrapper);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Status request (1.8.9)"
    }
}

impl PacketHandler<FalconConnection> for StatusPingPacket {
    fn handle_packet(self, connection: &mut FalconConnection) -> TaskScheduleResult {
        trace!("Sent status pong");
        falcon_send::send_status_pong(self.payload, connection);
        connection
            .handler_state_mut()
            .set_connection_state(ConnectionState::Disconnected);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Ping Request (1.8.9)"
    }
}
