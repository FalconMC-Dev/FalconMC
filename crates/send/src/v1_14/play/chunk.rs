falcon_send_derive::falcon_send! {
    use serde::Serialize;
    use fastnbt::LongArray;
    use falcon_core::network::buffer::{get_var_i32_size, PacketBufferWrite};
    use falcon_core::network::packet::PacketEncode;
    use falcon_core::world::blocks::Blocks;
    use falcon_core::world::chunks::{SECTION_HEIGHT, SECTION_LENGTH, SECTION_WIDTH};
    use falcon_core::world::palette::PaletteToI32;
    use crate::specs::play::{ChunkDataSpec, ChunkSectionDataSpec};
    use crate::util::HeightMap;
    use crate::v1_13::play::build_compacted_data_array;

    const MAX_BITS_PER_BLOCK: u8 = 14;

    const BIOME_COUNT: u16 = SECTION_WIDTH * SECTION_LENGTH;
    const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];

    #[falcon_packet(versions = {
        477, 480, 485, 490, 498 = 0x21;
    }, name = "chunk_data", batching = "build_chunk_data")]
    pub struct ChunkDataPacket {
        chunk_x: i32,
        chunk_z: i32,
        bit_mask: i32,
        heightmap: PacketHeightMap,
        chunk_sections: Vec<ChunkSectionData>,
    }

    #[derive(Serialize)]
    pub(crate) struct PacketHeightMap {
        #[serde(rename = "MOTION_BLOCKING")]
        motion_blocking: LongArray,
    }

    impl From<HeightMap> for PacketHeightMap {
        fn from(map: HeightMap) -> Self {
            PacketHeightMap {
                motion_blocking: LongArray::new(
                    build_compacted_data_array(
                        9,
                        36,
                        map.motion_blocking().into_iter().map(|v| v as u64)
                    ).into_iter().map(|v| v as i64).collect()
                )
            }
        }
    }

    impl PacketEncode for ChunkDataPacket {
        fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
            buf.write_i32(self.chunk_x);
            buf.write_i32(self.chunk_z);
            buf.write_bool(true); // We only send full chunks currently!
            buf.write_var_i32(self.bit_mask);
            buf.write_u8_array(fastnbt::to_bytes(&self.heightmap).unwrap().as_slice());
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
                heightmap: HeightMap::from_sections(&spec.sections, Blocks::get_global_id_1976).into(),
                chunk_sections: spec.sections.into_iter().map(|e| into_chunk_section(e, Blocks::get_global_id_1976)).collect(),
            }
        }
    }

    pub struct ChunkSectionData {
        pub(crate) block_count: i16,
        pub(crate) bits_per_block: u8,
        pub(crate) palette: Option<Vec<i32>>,
        pub(crate) block_data: Vec<u64>,
    }

    impl PacketEncode for ChunkSectionData {
        fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
            buf.write_i16(self.block_count);
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
        }
    }

    impl ChunkSectionData {
        pub fn get_data_size(&self) -> i32 {
            let mut size = 1 + 2; // always one for bits per block + block count: i16;
            if let Some(palette) = &self.palette {
                size += get_var_i32_size(palette.len() as i32);
                size += palette.iter().map(|x| get_var_i32_size(*x)).sum::<usize>();
            }
            size += get_var_i32_size(self.block_data.len() as i32);
            size += self.block_data.len() * std::mem::size_of::<u64>();
            size as i32
        }
    }

    pub(crate) fn into_chunk_section(spec: ChunkSectionDataSpec, block_to_int: PaletteToI32<Blocks>) -> ChunkSectionData {
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

        let mut block_count = 0;
        let block_iterator = spec.blocks.into_iter()
            .inspect(|v| {
                let block = spec.palette.at(*v as usize).unwrap();
                if !matches!(block, Blocks::Air | Blocks::VoidAir | Blocks::CaveAir) && block_to_int(block).is_some() {
                    block_count += 1;
                }
            });
        let (block_data, palette) = if bits_per_block > 8 {
            let blocks = spec.palette.build_direct_palette(block_iterator, block_to_int, Blocks::Air);
            let block_data = build_compacted_data_array(MAX_BITS_PER_BLOCK, (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16) as u32 / i64::BITS, blocks);
            (block_data, None)
        } else {
            let (blocks, palette) = spec.palette.build_indirect_palette(block_iterator, block_to_int, Blocks::Air);
            let block_data = build_compacted_data_array(bits_per_block, (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16) as u32 / i64::BITS, blocks);
            (block_data, Some(palette))
        };
        ChunkSectionData {
            block_count,
            bits_per_block,
            palette,
            block_data,
        }
    }
}
