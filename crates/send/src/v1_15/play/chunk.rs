#[falcon_send_derive::falcon_send]
mod inner {
    use crate::specs::play::ChunkDataSpec;
    use crate::util::HeightMap;
    use crate::v1_14::play::{into_chunk_section, ChunkSectionData, PacketHeightMap};
    use bytes::BufMut;
    use falcon_core::world::blocks::Blocks;
    use falcon_packet_core::{PacketIter, PacketSize, PacketWrite, WriteError};

    const BIOME_COUNT: u16 = 1024;
    const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];

    #[derive(PacketSize, PacketWrite)]
    #[falcon_packet(versions = {
        573, 575, 578 = 0x22;
    }, name = "chunk_data")]
    pub struct ChunkDataPacket {
        chunk_x: i32,
        chunk_z: i32,
        full_chunk: bool, // default true
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
        block_entity_num: i32, // default 0
    }

    #[inline(always)]
    #[allow(clippy::ptr_arg)]
    pub(crate) fn data_value(field: &[ChunkSectionData]) -> usize {
        data_size(field)
    }

    #[allow(clippy::ptr_arg)]
    pub(crate) fn data_size(field: &[ChunkSectionData]) -> usize {
        PacketIter::new(field.iter()).size_ref()
    }

    pub(crate) fn data_write<B: BufMut + ?Sized>(
        field: &[ChunkSectionData],
        buffer: &mut B,
    ) -> Result<(), WriteError> {
        PacketIter::new(field.iter()).write_ref(buffer)
    }

    impl From<ChunkDataSpec> for ChunkDataPacket {
        fn from(spec: ChunkDataSpec) -> Self {
            ChunkDataPacket {
                chunk_x: spec.chunk_x,
                chunk_z: spec.chunk_z,
                full_chunk: true,
                bitmask: spec.bitmask,
                heightmap: HeightMap::from_sections(&spec.sections, Blocks::get_global_id_2230)
                    .into(),
                biomes: BIOMES,
                size: 0,
                sections: spec
                    .sections
                    .into_iter()
                    .map(|e| into_chunk_section(e, Blocks::get_global_id_2230))
                    .collect(),
                block_entity_num: 0,
            }
        }
    }
}
