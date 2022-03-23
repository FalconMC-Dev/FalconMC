pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use uuid::Uuid;
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::ConnectionState;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, TaskScheduleResult};
    use falcon_core::server::ServerActor;

    #[derive(PacketDecode)]
    #[falcon_packet(47 = 0x01)]
    pub struct LoginStartPacket {
        name: String,
    }

    impl PacketHandler for LoginStartPacket {
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> TaskScheduleResult {
            debug!(player_name = %self.name);
            let player_uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, self.name.as_bytes());
            // TODO: more idiomatic code pls
            /*connection.send_packet(
                2,
                &LoginSuccessPacket {
                    uuid: player_uuid.to_hyphenated_ref().to_string(),
                    username: self.name.clone(),
                },
            );*/
            let handler_state = connection.handler_state_mut();
            handler_state.set_connection_state(ConnectionState::Play);
            handler_state.set_player_uuid(player_uuid);

            let version = connection.handler_state().protocol_id();
            let wrapper = connection.wrapper();
            connection.server().player_join(self.name, player_uuid, version, wrapper);
            Ok(())
        }

        fn get_name(&self) -> &'static str {
            "Login Start (1.8.9)"
        }
    }
}