pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use uuid::Uuid;
    use falcon_core::network::packet::PacketEncode;
    use crate::LoginSuccessSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(735 = 0x02; no_receive; outgoing = "login_success")]
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