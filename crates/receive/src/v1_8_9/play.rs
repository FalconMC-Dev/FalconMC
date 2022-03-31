pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::connection::ClientConnection;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_logic::ServerLogic;

    #[derive(PacketDecode)]
    #[falcon_packet(47 = 0x04; 107, 108, 109, 110, 210, 315, 316 = 0x0C; 335 = 0x0E; 338, 340 = 0x0D; 393, 401, 404 = 0x10)]
    pub struct PlayerPositionPacket {
        x: f64,
        y: f64,
        z: f64,
        on_ground: bool,
    }

    impl PacketHandler for PlayerPositionPacket {
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
            if let Some(uuid) = connection.handler_state().player_uuid() {
                connection.server()
                    .player_update_pos_look(uuid, Some(self.x), Some(self.y), Some(self.z), None, None, self.on_ground);
            }
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Player Position (1.8.9)"
        }
    }

    #[derive(PacketDecode)]
    #[falcon_packet(47 = 0x05; 107, 108, 109, 110, 210, 315, 316 = 0x0E; 335 = 0x10; 338, 340 = 0x0F; 393, 401, 404 = 0x12)]
    pub struct PlayerLookPacket {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    }

    impl PacketHandler for PlayerLookPacket {
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
            let uuid = connection.handler_state().player_uuid().expect("Something impossible happened");
            connection.server()
                .player_update_pos_look(uuid, None, None, None, Some(self.yaw), Some(self.pitch), self.on_ground);
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Player Look (1.8.9)"
        }
    }

    #[derive(PacketDecode)]
    #[falcon_packet(47 = 0x06; 107, 108, 109, 110, 210, 315, 316 = 0x0D; 335 = 0x0F; 338, 340 = 0x0E; 393, 401, 404 = 0x11)]
    pub struct PositionLookPacket {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    }

    impl PacketHandler for PositionLookPacket {
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
            let uuid = connection.handler_state().player_uuid().expect("Something impossible happened");
            connection.server()
                .player_update_pos_look(uuid, Some(self.x), Some(self.y), Some(self.z), Some(self.yaw), Some(self.pitch), self.on_ground);
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Position And Look (1.8.9)"
        }
    }
}