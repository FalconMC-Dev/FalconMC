falcon_send_derive::falcon_send! {
    use uuid::Uuid;
    use falcon_core::network::packet::PacketEncode;
    use crate::LoginSuccessSpec;

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
