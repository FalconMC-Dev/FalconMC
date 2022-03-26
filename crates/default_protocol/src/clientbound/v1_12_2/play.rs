pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::packet::PacketEncode;

    #[derive(PacketEncode)]
    #[falcon_packet(340 = 0x1F; 393, 401, 404 = 0x21; no_receive; outgoing = "keep_alive")]
    pub struct KeepAlivePacket {
        id: i64,
    }

    impl From<i64> for KeepAlivePacket {
        fn from(id: i64) -> Self {
            KeepAlivePacket {
                id
            }
        }
    }
}