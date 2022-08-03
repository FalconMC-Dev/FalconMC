falcon_send_derive::falcon_send! {
    use falcon_core::network::buffer::PacketBufferWrite;
    use falcon_core::network::packet::PacketEncode;
    use falcon_core::world::blocks::Blocks;
    use crate::specs::play::ChunkDataSpec;
    use crate::util::HeightMap;
    use crate::v1_14::play::{ChunkSectionData, into_chunk_section, PacketHeightMap};

    const BIOME_COUNT: u16 = 1024;
    const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];

    #[falcon_packet(versions = {
        573, 575, 578 = 0x22;
    }, name = "chunk_data", batching = "build_chunk_data")]
    pub struct ChunkDataPacket {
        chunk_x: i32,
        chunk_z: i32,
        bit_mask: i32,
        heightmap: PacketHeightMap,
        chunk_sections: Vec<ChunkSectionData>,
    }

    impl PacketEncode for ChunkDataPacket {
        fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
            buf.write_i32(self.chunk_x);
            buf.write_i32(self.chunk_z);
            buf.write_bool(true); // We only send full chunks currently!
            buf.write_var_i32(self.bit_mask);
            buf.write_u8_array(fastnbt::to_bytes(&self.heightmap).unwrap().as_slice());
            for x in BIOMES {
                buf.write_i32(x);
            }
            let data_size = self.chunk_sections.iter().map(|c| c.get_data_size()).sum();
            buf.write_var_i32(data_size);
            for chunk in &self.chunk_sections {
                chunk.to_buf(buf);
            }
            buf.write_var_i32(0);
        }
    }

    impl From<ChunkDataSpec> for ChunkDataPacket {
        fn from(spec: ChunkDataSpec) -> Self {
            ChunkDataPacket {
                chunk_x: spec.chunk_x,
                chunk_z: spec.chunk_z,
                bit_mask: spec.bitmask,
                heightmap: HeightMap::from_sections(&spec.sections, Blocks::get_global_id_2230).into(),
                chunk_sections: spec.sections.into_iter().map(|e| into_chunk_section(e, Blocks::get_global_id_2230)).collect(),
            }
        }
    }
}
