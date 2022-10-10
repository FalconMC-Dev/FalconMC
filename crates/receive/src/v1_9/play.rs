#[falcon_receive_derive::falcon_receive]
mod inner {
    use falcon_logic::{FalconConnection, connection::handler::PacketHandler};
    use falcon_packet_core::PacketRead;

    use crate::ReceiveError;

    #[derive(PacketRead)]
    #[falcon_packet(versions = {
        107, 108, 109, 110, 210, 315, 316, 393, 338, 340, 401, 404 = 0x04;
        335, 477, 480, 485, 490, 498, 573, 575, 578, 735, 736 = 0x05;
    })]
    pub struct ClientSettingsPacket {
        #[falcon(string = 16)]
        _locale: String,
        view_distance: u8,
        #[falcon(var32)]
        _chat_mode: i32,
        _chat_colors: bool,
        _skin_parts: u8,
        #[falcon(var32)]
        _main_hand: i32,
    }

    impl PacketHandler for ClientSettingsPacket {
        type Error = ReceiveError;

        fn handle_packet(self, connection: &mut FalconConnection) -> Result<(), Self::Error> {
            let uuid = connection.state().uuid.ok_or(ReceiveError::PlayerNotFound)?;
            connection.server().player_update_view_distance(uuid, self.view_distance);
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Client Settings (1.9)"
        }
    }
}
