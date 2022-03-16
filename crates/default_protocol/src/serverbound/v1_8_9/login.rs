pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use uuid::Uuid;
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::ConnectionState;
    use falcon_core::network::packet::{PacketDecode, PacketHandler, PacketHandlerError, PacketHandlerResult};
    use falcon_core::server::MinecraftServer;

    #[derive(PacketDecode)]
    #[falcon_packet(47 = 0x01)]
    pub struct LoginStartPacket {
        name: String,
    }

    impl PacketHandler for LoginStartPacket {
        fn handle_packet(self, connection: &mut dyn MinecraftConnection) -> PacketHandlerResult {
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
            let server_task = {
                let name = self.name;
                let version = connection.handler_state().protocol_id();
                let channel = connection.connection_link();
                Box::new(move |server: &mut dyn MinecraftServer| {
                    server.player_join(name, player_uuid, version, channel);
                })
            };
            connection
                .server_link_mut()
                .send(server_task)
                .map_err(|_| PacketHandlerError::ServerThreadSendError)
        }

        fn get_name(&self) -> &'static str {
            "Login Start (1.8.9)"
        }
    }
}