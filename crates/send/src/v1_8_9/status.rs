#[falcon_send_derive::falcon_send]
mod inner {
    use crate::specs::status::StatusResponseSpec;
    use falcon_core::network::packet::PacketEncode;
    use falcon_packet_core::{PacketSize, PacketWrite};

    #[derive(PacketEncode, PacketSize, PacketWrite)]
    #[falcon_packet(versions = { -1 = 0x00 }, name = "status_response")]
    pub struct StatusResponsePacket {
        #[max_length(32767)]
        #[falcon(string = 32767)]
        response: String,
    }

    impl From<StatusResponseSpec> for StatusResponsePacket {
        fn from(spec: StatusResponseSpec) -> Self {
            StatusResponsePacket {
                response: serde_json::to_string(&spec).expect("Invalid status data"),
            }
        }
    }

    #[derive(PacketEncode, PacketSize, PacketWrite)]
    #[falcon_packet(versions = { -1 = 0x01 }, name = "status_pong")]
    pub struct StatusPongPacket {
        payload: i64,
    }

    impl From<i64> for StatusPongPacket {
        fn from(payload: i64) -> Self {
            StatusPongPacket { payload }
        }
    }
}
