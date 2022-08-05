use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic};
use falcon_logic::FalconConnection;

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

impl<D: ConnectionDriver + 'static> PacketHandler<D, FalconConnection<D>> for StatusRequestPacket {
    fn handle_packet(self, connection: &mut FalconConnection<D>) -> TaskScheduleResult {
        let version = connection.driver().handler_state().protocol_id();
        let wrapper = connection.wrapper();
        connection.server().request_status(version, wrapper);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Status request (1.8.9)"
    }
}

impl<D: ConnectionDriver + 'static> PacketHandler<D, FalconConnection<D>> for StatusPingPacket {
    fn handle_packet(self, connection: &mut FalconConnection<D>) -> TaskScheduleResult {
        falcon_send::send_status_pong(self.payload, connection);
        connection
            .driver_mut()
            .handler_state_mut()
            .set_connection_state(ConnectionState::Disconnected);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Ping Request (1.8.9)"
    }
}
