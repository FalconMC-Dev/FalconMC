pub use inner::*;
pub use chunk::*;

pub mod dimension;
mod chunk;

#[falcon_protocol_derive::packet_module]
mod inner {
    use std::path::PathBuf;
    use falcon_core::data::Identifier;
    use falcon_core::network::buffer::PacketBufferWrite;
    use falcon_core::network::packet::PacketEncode;
    use falcon_core::world::dimension::Dimension;
    use crate::JoinGameSpec;
    use crate::v1_16::play::dimension::{Codec, DimensionData};

    #[falcon_packet(735 = 0x25; no_receive; outgoing = "join_game")]
    pub struct JoinGamePacket {
        spec: JoinGameSpec,
    }

    impl From<JoinGameSpec> for JoinGamePacket {
        fn from(spec: JoinGameSpec) -> Self {
            JoinGamePacket {
                spec,
            }
        }
    }

    impl PacketEncode for JoinGamePacket {
        fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
            self.spec.entity_id.to_buf(buf);
            (self.spec.game_mode as u8).to_buf(buf);
            (self.spec.game_mode as u8).to_buf(buf); // previous gamemode
            buf.write_var_i32(1); // world count
            let world_name = Identifier::from_static("falcon", "world");
            world_name.to_buf(buf); // worlds
            let codec = Codec::new(vec![
                DimensionData::new(Dimension::new("minecraft:overworld", 0)),
            ]);
            buf.write_u8_array(&fastnbt::ser::to_bytes(&codec).unwrap()); // Dimension codec
            Identifier::from_static("minecraft", "overworld").to_buf(buf); // dimension
            world_name.to_buf(buf); // world name
            self.spec.hashed_seed.to_buf(buf);
            self.spec.max_players.to_buf(buf);
            buf.write_var_i32(self.spec.view_distance);
            self.spec.reduced_debug.to_buf(buf);
            self.spec.enable_respawn_screen.to_buf(buf);
            buf.write_bool(false);
            buf.write_bool(false);
        }
    }
}