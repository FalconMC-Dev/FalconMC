pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::connection::ClientConnection;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_logic::ServerLogic;

    #[derive(PacketDecode)]
    #[falcon_packet(107, 108, 109, 110, 210, 315, 316, 393, 338, 340, 401, 404 = 0x04; 335, 477, 480, 485, 490, 498, 573, 575, 578 = 0x05)]
    pub struct ClientSettingsPacket {
        #[max_length(16)]
        _locale: String,
        view_distance: u8,
        #[var_int]
        _chat_mode: i32,
        _chat_colors: bool,
        _skin_parts: u8,
        #[var_int]
        _main_hand: i32,
    }

    impl PacketHandler for ClientSettingsPacket {
        fn handle_packet(self, connection: &mut ClientConnection) -> TaskScheduleResult {
            if let Some(uuid) = connection.handler_state().player_uuid() {
                connection.server()
                    .player_update_view_distance(uuid, self.view_distance);
            }
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Client Settings (1.9)"
        }
    }
}