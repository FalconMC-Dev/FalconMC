falcon_send_derive::falcon_send! {
    use mc_chat::ChatComponent;
    use falcon_core::network::packet::PacketEncode;
    use crate::specs::login::LoginSuccessSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(versions = { -1 = 0x00 }, name = "disconnect")]
    pub struct DisconnectPacket {
        #[max_length(262144)]
        reason: String,
    }

    impl From<ChatComponent> for DisconnectPacket {
        fn from(reason: ChatComponent) -> Self {
            DisconnectPacket {
                reason: serde_json::to_string(&reason).unwrap(),
            }
        }
    }

    #[derive(PacketEncode)]
    #[falcon_packet(
        versions = {
            47, 393, 401, 404, 477, 480, 485, 490, 498, 573, 575, 578 = 0x02;
        }, name = "login_success"
    )]
    pub struct LoginSuccess {
        uuid: String,
        #[max_length(16)]
        username: String,
    }

    impl From<LoginSuccessSpec> for LoginSuccess {
        fn from(spec: LoginSuccessSpec) -> Self {
            LoginSuccess {
                uuid: spec.uuid.as_hyphenated().to_string(),
                username: spec.username,
            }
        }
    }
}
