falcon_send_derive::falcon_send! {
    use falcon_core::network::packet::PacketEncode;

    #[derive(PacketEncode)]
    #[falcon_packet(versions = {
        340 = 0x1F;
        393, 401, 404, 573, 575, 578 = 0x21;
        477, 480, 485, 490, 498, 735, 736 = 0x20;
    }, name = "keep_alive")]
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
