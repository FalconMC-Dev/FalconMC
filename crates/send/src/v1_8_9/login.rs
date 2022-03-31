pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use mc_chat::ChatComponent;
    use falcon_core::network::packet::PacketEncode;
    use crate::specs::login::LoginSuccessSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(-1 = 0x00; no_receive; outgoing = "disconnect")]
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
    #[falcon_packet(47, 393, 401, 404 = 0x02; outgoing = "login_success"; no_receive)]
    pub struct LoginSuccess {
        uuid: String,
        #[max_length(16)]
        username: String,
    }

    impl From<LoginSuccessSpec> for LoginSuccess {
        fn from(spec: LoginSuccessSpec) -> Self {
            LoginSuccess {
                uuid: spec.uuid.to_hyphenated_ref().to_string(),
                username: spec.username,
            }
        }
    }
}