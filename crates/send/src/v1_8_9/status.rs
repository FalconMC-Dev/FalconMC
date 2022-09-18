use falcon_send_derive::falcon_send;

#[falcon_send]
mod inner {
    use crate::specs::status::StatusResponseSpec;
    use falcon_core::network::packet::PacketEncode;

    #[derive(PacketEncode)]
    #[falcon_packet(versions = { -1 = 0x00 }, name = "status_response")]
    pub struct StatusResponsePacket {
        #[max_length(32767)]
        response: String,
    }

    impl From<StatusResponseSpec> for StatusResponsePacket {
        fn from(spec: StatusResponseSpec) -> Self {
            StatusResponsePacket {
                response: serde_json::to_string(&spec).expect("Invalid status data"),
            }
        }
    }

    #[derive(PacketEncode)]
    #[falcon_packet(versions = { -1 = 0x01 }, name = "status_pong")]
    pub struct StatusPongPacket {
        payload: i64,
    }

    impl From<i64> for StatusPongPacket {
        fn from(payload: i64) -> Self {
            StatusPongPacket {
                payload,
            }
        }
    }
}
