#[falcon_send_derive::falcon_send]
mod inner {
    use crate::LoginSuccessSpec;
    use falcon_core::network::packet::PacketEncode;
    use uuid::Uuid;

    #[derive(PacketEncode)]
    #[falcon_packet(versions = {
        735, 736 = 0x02;
    }, name = "login_success")]
    pub struct LoginSuccessPacket {
        uuid: Uuid,
        #[max_length(16)]
        username: String,
    }

    impl From<LoginSuccessSpec> for LoginSuccessPacket {
        fn from(spec: LoginSuccessSpec) -> Self {
            LoginSuccessPacket {
                uuid: spec.uuid,
                username: spec.username,
            }
        }
    }
}
