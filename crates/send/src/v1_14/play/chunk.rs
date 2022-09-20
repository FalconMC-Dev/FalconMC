#[falcon_send_derive::falcon_send]
mod inner {
    use crate::specs::play::{ChunkDataSpec, ChunkSectionDataSpec};
    use crate::util::HeightMap;
    use crate::v1_13::play::build_compacted_data_array;
    use bytes::BufMut;
    use falcon_core::world::blocks::Blocks;
    use falcon_core::world::chunks::{SECTION_HEIGHT, SECTION_LENGTH, SECTION_WIDTH};
    use falcon_core::world::palette::PaletteToI32;
    use falcon_packet_core::{
        PacketArray, PacketSize, PacketSizeSeed, PacketVec, PacketWrite, PacketWriteSeed, VarI32,
        WriteError,
    };
    use fastnbt::LongArray;
    use serde::Serialize;

    const MAX_BITS_PER_BLOCK: u8 = 14;

    const BIOME_COUNT: u16 = SECTION_WIDTH * SECTION_LENGTH;
    const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];

    #[derive(PacketSize, PacketWrite)]
    #[falcon_packet(versions = {
        477, 480, 485, 490, 498 = 0x21;
    }, name = "chunk_data", batching = "build_chunk_data")]
    pub struct ChunkDataPacket {
        chunk_x: i32,
        chunk_z: i32,
        full_chunk: bool, // default true
        #[falcon(var32)]
        bitmask: i32,
        #[falcon(nbt)]
        heightmap: PacketHeightMap,
        #[falcon(var32)]
        size: usize,
        #[falcon(link = "size with data")]
        sections: Vec<ChunkSectionData>,
        #[falcon(var32)]
        block_entity_num: i32, // default 0
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
                        map.motion_blocking().into_iter().map(|v| v as u64),
                    )
                    .into_iter()
                    .map(|v| v as i64)
                    .collect(),
                ),
            }
        }
    }

    #[inline(always)]
    pub(crate) fn data_value(field: &Vec<ChunkSectionData>) -> usize {
        data_size(field)
    }

    pub(crate) fn data_size(field: &Vec<ChunkSectionData>) -> usize {
        PacketSizeSeed::size(&PacketVec::default(), field) + BIOME_COUNT as usize * 4
    }

    pub(crate) fn data_write<B: BufMut + ?Sized>(
        field: Vec<ChunkSectionData>,
        buffer: &mut B,
    ) -> Result<(), WriteError> {
        PacketWriteSeed::write(PacketVec::default(), field, buffer)?;
        PacketWriteSeed::write(PacketArray::default(), BIOMES, buffer)
    }

    impl From<ChunkDataSpec> for ChunkDataPacket {
        fn from(spec: ChunkDataSpec) -> Self {
            ChunkDataPacket {
                chunk_x: spec.chunk_x,
                chunk_z: spec.chunk_z,
                full_chunk: true,
                bitmask: spec.bitmask,
                heightmap: HeightMap::from_sections(&spec.sections, Blocks::get_global_id_1976)
                    .into(),
                size: 0,
                sections: spec
                    .sections
                    .into_iter()
                    .map(|e| into_chunk_section(e, Blocks::get_global_id_1976))
                    .collect(),
                block_entity_num: 0,
            }
        }
    }

    pub struct ChunkSectionData {
        pub(crate) block_count: i16,
        pub(crate) bits_per_block: u8,
        pub(crate) palette: Option<Vec<i32>>,
        pub(crate) block_data: Vec<u64>,
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
            self.block_count.size()
                + self.bits_per_block.size()
                + palette_len
                + VarI32::from(self.block_data.len()).size()
                + self.block_data.len() * std::mem::size_of::<u64>()
        }
    }

    impl PacketWrite for ChunkSectionData {
        fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
        where
            B: BufMut + ?Sized,
        {
            self.block_count.write(buffer)?;
            self.bits_per_block.write(buffer)?;
            if let Some(palette) = self.palette {
                VarI32::from(palette.len()).write(buffer)?;
                PacketWriteSeed::write(
                    PacketVec::default(),
                    palette.into_iter().map(|x| VarI32::from(x)).collect(),
                    buffer,
                )?;
            }
            VarI32::from(self.block_data.len()).write(buffer)?;
            PacketWriteSeed::write(PacketVec::default(), self.block_data, buffer)
        }
    }

    pub(crate) fn into_chunk_section(
        spec: ChunkSectionDataSpec,
        block_to_int: PaletteToI32<Blocks>,
    ) -> ChunkSectionData {
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
        let block_iterator = spec.blocks.into_iter().inspect(|v| {
            let block = spec.palette.at(*v as usize).unwrap();
            if !matches!(block, Blocks::Air | Blocks::VoidAir | Blocks::CaveAir)
                && block_to_int(block).is_some()
            {
                block_count += 1;
            }
        });
        let (block_data, palette) = if bits_per_block > 8 {
            let blocks =
                spec.palette
                    .build_direct_palette(block_iterator, block_to_int, Blocks::Air);
            let block_data = build_compacted_data_array(
                MAX_BITS_PER_BLOCK,
                (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16) as u32
                    / i64::BITS,
                blocks,
            );
            (block_data, None)
        } else {
            let (blocks, palette) =
                spec.palette
                    .build_indirect_palette(block_iterator, block_to_int, Blocks::Air);
            let block_data = build_compacted_data_array(
                bits_per_block,
                (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16) as u32
                    / i64::BITS,
                blocks,
            );
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
