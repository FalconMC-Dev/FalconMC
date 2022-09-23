#[falcon_send_derive::falcon_send]
mod inner {
    use crate::util::HeightMap;
    use crate::v1_14::play::ChunkSectionData;
    use crate::{ChunkDataSpec, ChunkSectionDataSpec};
    use bytes::BufMut;
    use falcon_core::world::blocks::Blocks;
    use falcon_core::world::chunks::{SECTION_HEIGHT, SECTION_LENGTH, SECTION_WIDTH};
    use falcon_core::world::palette::PaletteToI32;
    use falcon_packet_core::{
        PacketSize, PacketSizeSeed, PacketVec, PacketWrite, PacketWriteSeed, WriteError,
    };
    use fastnbt::LongArray;
    use serde::Serialize;

    const MAX_BITS_PER_BLOCK: u8 = 15;

    const BIOME_COUNT: u16 = 1024;
    const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];

    #[derive(PacketSize, PacketWrite)]
    #[falcon_packet(versions = {
        735, 736 = 0x21;
    }, name = "chunk_data")]
    pub struct ChunkDataPacket {
        chunk_x: i32,
        chunk_z: i32,
        full_chunk: bool,
        ignore_old: bool,
        #[falcon(var32)]
        bitmask: i32,
        #[falcon(nbt)]
        heightmap: PacketHeightMap,
        #[falcon(array)]
        biomes: [i32; BIOME_COUNT as usize],
        #[falcon(var32)]
        size: usize,
        #[falcon(link = "size with data")]
        sections: Vec<ChunkSectionData>,
        #[falcon(var32)]
        block_entity_num: i32,
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
                        37,
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
    #[allow(clippy::ptr_arg)]
    pub(crate) fn data_value(field: &Vec<ChunkSectionData>) -> usize {
        data_size(field)
    }

    #[allow(clippy::ptr_arg)]
    pub(crate) fn data_size(field: &Vec<ChunkSectionData>) -> usize {
        PacketSizeSeed::size(&PacketVec::default(), field)
    }

    pub(crate) fn data_write<B: BufMut + ?Sized>(
        field: Vec<ChunkSectionData>,
        buffer: &mut B,
    ) -> Result<(), WriteError> {
        PacketWriteSeed::write(PacketVec::default(), field, buffer)
    }

    impl From<ChunkDataSpec> for ChunkDataPacket {
        fn from(spec: ChunkDataSpec) -> Self {
            ChunkDataPacket {
                chunk_x: spec.chunk_x,
                chunk_z: spec.chunk_z,
                full_chunk: true,
                ignore_old: false,
                bitmask: spec.bitmask,
                heightmap: HeightMap::from_sections(&spec.sections, Blocks::get_global_id_2567)
                    .into(),
                biomes: BIOMES,
                size: 0,
                sections: spec
                    .sections
                    .into_iter()
                    .map(|e| into_chunk_section(e, Blocks::get_global_id_2567))
                    .collect(),
                block_entity_num: 0,
            }
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

    pub fn build_compacted_data_array<E: Iterator<Item = u64>>(
        bits_per_element: u8,
        capacity: u32,
        elements: E,
    ) -> Vec<u64> {
        let mut compacted_data = Vec::with_capacity(capacity as usize);
        let mut current_long = 0u64;

        let entries_per_long = (i64::BITS / bits_per_element as u32) as u8;
        let mut current_entry = 0u8;
        for element in elements {
            current_long |= element << (current_entry * bits_per_element);
            current_entry += 1;
            if current_entry == entries_per_long {
                current_entry = 0;
                compacted_data.push(current_long);
                current_long = 0u64;
            }
        }
        if current_entry != 0 {
            compacted_data.push(current_long);
        }

        compacted_data
    }
}
