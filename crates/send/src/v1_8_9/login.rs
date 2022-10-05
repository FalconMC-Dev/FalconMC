#[falcon_send_derive::falcon_send]
mod inner {
    use derive_from_ext::From;
    use falcon_packet_core::special::StrUuid;
    use falcon_packet_core::{PacketSize, PacketWrite};
    use mc_chat::ChatComponent;

    use crate::specs::login::LoginSuccessSpec;

    #[derive(PacketSize, PacketWrite)]
    #[falcon_packet(versions = { -1 = 0x00 }, name = "disconnect")]
    pub struct DisconnectPacket {
        #[falcon(string = 262144)]
        reason: String,
    }

    impl From<ChatComponent> for DisconnectPacket {
        fn from(reason: ChatComponent) -> Self {
            DisconnectPacket {
                reason: serde_json::to_string(&reason).unwrap(),
            }
        }
    }

    #[derive(PacketSize, PacketWrite, From)]
    #[from(LoginSuccessSpec)]
    #[falcon_packet(
        versions = {
            47, 393, 401, 404, 477, 480, 485, 490, 498, 573, 575, 578 = 0x02;
        }, name = "login_success"
    )]
    pub struct LoginSuccess {
        uuid: StrUuid,
        #[falcon(string = 16)]
        username: String,
    }
}
