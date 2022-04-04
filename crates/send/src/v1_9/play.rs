pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::packet::PacketEncode;
    use crate::specs::play::PositionAndLookSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(107, 108, 109, 110, 210, 315, 316, 335 = 0x2E; 338, 340 = 0x2F; 393, 401, 404 = 0x32; 477, 480, 485, 490, 498 = 0x35; no_receive; outgoing = "position_look")]
    pub struct PositionLookPacket {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: u8,
        #[var_int]
        teleport_id: i32,
    }

    impl From<PositionAndLookSpec> for PositionLookPacket {
        fn from(spec: PositionAndLookSpec) -> Self {
            PositionLookPacket {
                x: spec.x,
                y: spec.y,
                z: spec.z,
                yaw: spec.yaw,
                pitch: spec.pitch,
                flags: spec.flags,
                teleport_id: spec.teleport_id
            }
        }
    }

    #[derive(PacketEncode)]
    #[falcon_packet(107, 108, 109, 110, 210, 315, 316, 335, 338, 340, 477, 480, 485, 490, 498 = 0x1D; 393, 401, 404 = 0x1F; no_receive; outgoing = "unload_chunk")]
    pub struct UnloadChunkPacket {
        chunk_x: i32,
        chunk_z: i32,
    }

    impl From<(i32, i32)> for UnloadChunkPacket {
        fn from((chunk_x, chunk_z): (i32, i32)) -> Self {
            UnloadChunkPacket {
                chunk_x,
                chunk_z
            }
        }
    }
}