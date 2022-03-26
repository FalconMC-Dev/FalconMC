pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::buffer::{get_var_i32_size, PacketBufferWrite};
    use falcon_core::network::packet::PacketEncode;
    use falcon_core::world::blocks::Blocks;
    use falcon_core::world::chunks::{SECTION_HEIGHT, SECTION_LENGTH, SECTION_WIDTH};
    use crate::clientbound::specs::play::{ChunkDataSpec, ChunkSectionDataSpec};

    const MAX_BITS_PER_BLOCK: u8 = 14;

    const BIOME_COUNT: u16 = SECTION_WIDTH * SECTION_LENGTH;
    const LIGHT_COUNT: usize = ((SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH) / 2) as usize;
    const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];
    const MAX_LIGHT: [u8; LIGHT_COUNT] = [0xFF; LIGHT_COUNT];

    #[falcon_packet(393, 401, 404 = 0x22; no_receive; outgoing = "chunk_data")]
    pub struct ChunkDataPacket {
        chunk_x: i32,
        chunk_z: i32,
        bit_mask: i32,
        chunk_sections: Vec<ChunkSectionData>,
    }

    impl PacketEncode for ChunkDataPacket {
        fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
            buf.write_i32(self.chunk_x);
            buf.write_i32(self.chunk_z);
            buf.write_bool(true); // We only send full chunks currently!
            buf.write_var_i32(self.bit_mask);
            let mut data_size: i32 = BIOME_COUNT as i32 * 4; // biomes get sent because of full chunk
            for chunk in &self.chunk_sections {
                data_size += chunk.get_data_size();
            }
            buf.write_var_i32(data_size);
            for chunk in &self.chunk_sections {
                chunk.to_buf(buf);
            }
            for x in BIOMES {
                buf.write_i32(x);
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
                chunk_sections: spec.sections.into_iter().map(|e| e.into()).collect(),
            }
        }
    }

    pub struct ChunkSectionData {
        bits_per_block: u8,
        palette: Option<Vec<i32>>,
        block_data: Vec<u64>,
    }

    impl PacketEncode for ChunkSectionData {
        fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
            buf.write_u8(self.bits_per_block);
            if let Some(palette) = &self.palette {
                buf.write_var_i32(palette.len() as i32);
                for x in palette {
                    buf.write_var_i32(*x);
                }
            }
            buf.write_var_i32(self.block_data.len() as i32);
            for x in &self.block_data {
                buf.write_i64(*x as i64);
            }
            for x in MAX_LIGHT {
                buf.write_u8(x);
            }
            for x in MAX_LIGHT {
                buf.write_u8(x);
            }
        }
    }

    impl ChunkSectionData {
        pub fn get_data_size(&self) -> i32 {
            let mut size = 1; // always one for bits per block;
            if let Some(palette) = &self.palette {
                size += get_var_i32_size(palette.len() as i32);
                size += palette.iter().map(|x| get_var_i32_size(*x)).sum::<usize>();
            }
            size += get_var_i32_size(self.block_data.len() as i32);
            size += self.block_data.len() * std::mem::size_of::<u64>();
            size += LIGHT_COUNT;
            size += LIGHT_COUNT; // we only have the overworld for now
            size as i32
        }
    }

    impl From<ChunkSectionDataSpec> for ChunkSectionData {
        fn from(mut spec: ChunkSectionDataSpec) -> Self {
            let block_to_int = match spec.protocol_version {
                401 | 404 => Blocks::get_global_id_1631,
                _ => Blocks::get_global_id_1519,
            };
            let bits_per_block = {
                let actual = spec.palette.calculate_bits_per_entry(block_to_int);
                if actual < 4 {
                    4u8
                } else if actual < 9 {
                    actual as u8
                } else {
                    MAX_BITS_PER_BLOCK
                }
            };

            let (block_data, palette) = if bits_per_block > 8 {
                let blocks = spec.palette.build_direct_palette(spec.blocks.drain(..), block_to_int, Blocks::Air);
                let block_data = build_compacted_data_array(MAX_BITS_PER_BLOCK, blocks);
                (block_data, None)
            } else {
                let (blocks, palette) = spec.palette.build_indirect_palette(spec.blocks.drain(..), block_to_int, Blocks::Air);
                let block_data = build_compacted_data_array(bits_per_block, blocks);
                (block_data, Some(palette))
            };
            ChunkSectionData {
                bits_per_block,
                palette,
                block_data,
            }
        }
    }

    pub fn build_compacted_data_array<E: Iterator<Item=u64>>(bits_per_block: u8, elements: E) -> Vec<u64> {
        let long_count: u32 = (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16) as u32 / i64::BITS;
        let mut compacted_data = Vec::with_capacity(long_count as usize);
        let mut current_long = 0u64;
        let mut offset = 0;
        let mut pos = 0;

        for element in elements {
            let bit_shift = pos * bits_per_block + offset;
            if bit_shift < (i64::BITS - bits_per_block as u32) as u8 {
                current_long |= element << bit_shift;
                pos += 1;
            } else {
                offset = bit_shift - (i64::BITS - bits_per_block as u32) as u8;
                current_long |= element << bit_shift;
                compacted_data.push(current_long);
                current_long = 0u64;
                if offset != 0 {
                    let diff = bits_per_block - offset;
                    current_long |= element >> diff;
                }
                pos = 0;
            }
        }

        compacted_data
    }
}