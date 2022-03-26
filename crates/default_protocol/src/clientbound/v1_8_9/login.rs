pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::packet::PacketEncode;
    use crate::clientbound::specs::login::LoginSuccessSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(47, 404 = 0x02; outgoing = "login_success"; no_receive)]
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