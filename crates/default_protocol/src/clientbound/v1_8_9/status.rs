pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::packet::PacketEncode;
    use crate::clientbound::specs::status::StatusResponseSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(-1 = 0x00; no_receive; outgoing = "status_response")]
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
    #[falcon_packet(-1 = 0x01; no_receive; outgoing = "status_pong")]
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