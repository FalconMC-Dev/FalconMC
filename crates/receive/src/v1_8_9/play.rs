use falcon_core::network::connection::ConnectionLogic;
use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
use falcon_core::player::data::Position;
use falcon_logic::FalconConnection;

falcon_receive_derive::falcon_receive! {
    #[derive(PacketDecode)]
    #[falcon_packet(versions = {
        47 = 0x04;
        107, 108, 109, 110, 210, 315, 316 = 0x0C;
        335 = 0x0E;
        338, 340 = 0x0D;
        393, 401, 404 = 0x10;
        477, 480, 485, 490, 498, 573, 575, 578 = 0x11;
        735, 736 = 0x12;
    })]
    pub struct PlayerPositionPacket {
        x: f64,
        y: f64,
        z: f64,
        on_ground: bool,
    }

    #[derive(PacketDecode)]
    #[falcon_packet(versions = {
        47 = 0x05;
        107, 108, 109, 110, 210, 315, 316 = 0x0E;
        335 = 0x10;
        338, 340 = 0x0F;
        393, 401, 404 = 0x12;
        477, 480, 485, 490, 498, 573, 575, 578 = 0x13;
        735, 736 = 0x14;
    })]
    pub struct PlayerLookPacket {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    }

    #[derive(PacketDecode)]
    #[falcon_packet(versions = {
        47 = 0x06;
        107, 108, 109, 110, 210, 315, 316 = 0x0D;
        335 = 0x0F;
        338, 340 = 0x0E;
        393, 401, 404 = 0x11;
        477, 480, 485, 490, 498, 573, 575, 578 = 0x12;
        735, 736 = 0x13;
    })]
    pub struct PositionLookPacket {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    }
}

impl PacketHandler<FalconConnection> for PlayerPositionPacket {
    fn handle_packet(self, connection: &mut FalconConnection) -> TaskScheduleResult {
        if let Some(uuid) = connection.handler_state().player_uuid() {
            connection.server()
                .player_update_pos_look(uuid, Some(Position::new(self.x, self.y, self.z)), None, self.on_ground);
        }
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Player Position (1.8.9)"
    }
}

impl PacketHandler<FalconConnection> for PlayerLookPacket {
    fn handle_packet(self, connection: &mut FalconConnection) -> TaskScheduleResult {
        let uuid = connection.handler_state().player_uuid().expect("Something impossible happened");
        connection.server()
            .player_update_pos_look(uuid, None, Some((self.yaw, self.pitch)), self.on_ground);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Player Look (1.8.9)"
    }
}

impl PacketHandler<FalconConnection> for PositionLookPacket {
    fn handle_packet(self, connection: &mut FalconConnection) -> TaskScheduleResult {
        let uuid = connection.handler_state().player_uuid().expect("Something impossible happened");
        connection.server()
            .player_update_pos_look(uuid, Some(Position::new(self.x, self.y, self.z)), Some((self.yaw, self.pitch)), self.on_ground);
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Position And Look (1.8.9)"
    }
}
