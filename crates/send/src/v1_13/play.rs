#[falcon_send_derive::falcon_send]
mod inner {
    use crate::specs::play::{ChunkDataSpec, ChunkSectionDataSpec};
    use bytes::BufMut;
    use derive_from_ext::From;
    use falcon_core::world::blocks::Blocks;
    use falcon_core::world::chunks::{SECTION_HEIGHT, SECTION_LENGTH, SECTION_WIDTH};
    use falcon_packet_core::{
        PacketArray, PacketIter, PacketSize, PacketVec, PacketWrite, PacketWriteSeed, VarI32,
        WriteError,
    };

    const MAX_BITS_PER_BLOCK: u8 = 14;

    const BIOME_COUNT: u16 = SECTION_WIDTH * SECTION_LENGTH;
    const LIGHT_COUNT: usize = ((SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH) / 2) as usize;
    const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];
    const MAX_LIGHT: [u8; LIGHT_COUNT] = [0xFF; LIGHT_COUNT];

    #[derive(PacketSize, PacketWrite, From)]
    #[from(ChunkDataSpec)]
    #[falcon_packet(versions = {
        393, 401, 404 = 0x22;
    }, name = "chunk_data")]
    pub struct ChunkDataPacket {
        chunk_x: i32,
        chunk_z: i32,
        #[from(skip, default = "true")]
        full_chunk: bool,
        #[falcon(var32)]
        bitmask: i32,
        #[from(skip)]
        #[falcon(var32)]
        size: usize, // filled in by sections field
        #[from(map = "data_map")]
        #[falcon(link = "size with data")]
        sections: Vec<ChunkSectionData>,
        #[from(skip)]
        #[falcon(var32)]
        block_entity_num: i32, // default 0
    }

    fn data_map(sections: Vec<ChunkSectionDataSpec>) -> Vec<ChunkSectionData> {
        sections.into_iter().map(|s| s.into()).collect()
    }

    #[inline(always)]
    #[allow(clippy::ptr_arg)]
    fn data_value(field: &[ChunkSectionData]) -> usize {
        data_size(field)
    }

    #[allow(clippy::ptr_arg)]
    fn data_size(field: &[ChunkSectionData]) -> usize {
        PacketIter::new(field.iter()).size_ref() + BIOME_COUNT as usize * 4
    }

    fn data_write<B: BufMut + ?Sized>(
        field: &[ChunkSectionData],
        buffer: &mut B,
    ) -> Result<(), WriteError> {
        PacketIter::new(field.iter()).write_ref(buffer)?;
        PacketWriteSeed::write(PacketArray::default(), &BIOMES, buffer)
    }

    pub struct ChunkSectionData {
        bits_per_block: u8,
        palette: Option<Vec<i32>>,
        block_data: Vec<u64>,
    }

    impl PacketSize for ChunkSectionData {
        fn size(&self) -> usize {
            let palette_len = if let Some(palette) = &self.palette {
                VarI32::from(palette.len()).size()
                    + palette
                        .iter()
                        .map(|x| VarI32::from(*x).size())
                        .sum::<usize>()
            } else {
                0
            };
            self.bits_per_block.size()
            + palette_len
            + VarI32::from(self.block_data.len()).size()
            + self.block_data.len() * std::mem::size_of::<u64>()
            + LIGHT_COUNT // block light
            + LIGHT_COUNT // sky light
        }
    }

    impl PacketWrite for ChunkSectionData {
        fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
        where
            B: BufMut + ?Sized,
        {
            self.bits_per_block.write(buffer)?;
            if let Some(palette) = &self.palette {
                VarI32::from(palette.len() as i32).write(buffer)?;
                PacketIter::new(palette.iter().map(|&x| VarI32::from(x))).write_owned(buffer)?;
            }
            VarI32::from(self.block_data.len()).write(buffer)?;
            PacketWriteSeed::write(PacketVec::default(), &self.block_data, buffer)?;
            MAX_LIGHT.write(buffer)?;
            MAX_LIGHT.write(buffer)
        }
    }

    impl From<ChunkSectionDataSpec> for ChunkSectionData {
        fn from(spec: ChunkSectionDataSpec) -> Self {
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
                let blocks = spec.palette.build_direct_palette(
                    spec.blocks.into_iter(),
                    block_to_int,
                    Blocks::Air,
                );
                let block_data = build_compacted_data_array(
                    MAX_BITS_PER_BLOCK,
                    (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16)
                        as u32
                        / i64::BITS,
                    blocks,
                );
                (block_data, None)
            } else {
                let (blocks, palette) = spec.palette.build_indirect_palette(
                    spec.blocks.into_iter(),
                    block_to_int,
                    Blocks::Air,
                );
                let block_data = build_compacted_data_array(
                    bits_per_block,
                    (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16)
                        as u32
                        / i64::BITS,
                    blocks,
                );
                (block_data, Some(palette))
            };
            ChunkSectionData {
                bits_per_block,
                palette,
                block_data,
            }
        }
    }

    pub fn build_compacted_data_array<E: Iterator<Item = u64>>(
        bits_per_element: u8,
        capacity: u32,
        elements: E,
    ) -> Vec<u64> {
        let mut compacted_data = Vec::with_capacity(capacity as usize);
        let mut current_long = 0u64;
        let mut offset = 0;
        let mut pos = 0;

        for element in elements {
            let bit_shift = pos * bits_per_element + offset;
            if bit_shift < (i64::BITS - bits_per_element as u32) as u8 {
                current_long |= element << bit_shift;
                pos += 1;
            } else {
                offset = bit_shift - (i64::BITS - bits_per_element as u32) as u8;
                current_long |= element << bit_shift;
                compacted_data.push(current_long);
                current_long = 0u64;
                if offset != 0 {
                    let diff = bits_per_element - offset;
                    current_long |= element >> diff;
                }
                pos = 0;
            }
        }

        compacted_data
    }
}
