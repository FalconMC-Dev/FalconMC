pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_core::server::ServerActor;

    #[derive(PacketDecode)]
    #[falcon_packet(107, 108, 109, 110, 210, 315, 316, 393, 338, 340, 401, 404 = 0x04; 335 = 0x05)]
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
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> TaskScheduleResult {
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