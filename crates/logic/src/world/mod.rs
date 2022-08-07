use crate::player::FalconPlayer;
use crate::world::cache::WorldPacketCache;
use ahash::AHashMap;
use bytes::Bytes;
use falcon_core::error::FalconCoreError;
use falcon_core::network::buffer::read_var_i32_from_iter;
use falcon_core::schematic::SchematicData;
use falcon_core::world::blocks::Blocks;
use falcon_core::world::chunks::{Chunk, ChunkPos, SECTION_LENGTH, SECTION_WIDTH};
use falcon_send::specs::play::ChunkDataSpec;
use itertools::Itertools;

mod cache;

#[derive(Debug)]
pub struct FalconWorld {
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
    chunks: AHashMap<ChunkPos, Chunk>,
    cache: WorldPacketCache,
}

impl FalconWorld {
    pub fn new(capacity: usize, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> Self {
        FalconWorld {
            min_x,
            min_z,
            max_x,
            max_z,
            chunks: AHashMap::with_capacity(capacity),
            cache: Default::default(),
        }
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        if pos.x > self.max_x || pos.x < self.min_x || pos.z > self.max_z || pos.z < self.min_z {
            return None;
        }
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.chunks.entry(pos).or_insert_with(|| Chunk::empty(pos))
    }

    pub fn send_chunks_for_player(&mut self, player: &FalconPlayer) {
        let (chunk_x, chunk_z) = player.position().chunk_coords();
        let view_distance = player.view_distance();
        let capacity = (2 * view_distance as usize + 1).pow(2);
        let mut chunks = Vec::with_capacity(capacity);

        for x in chunk_x - view_distance as i32..=chunk_x + view_distance as i32 {
            for z in chunk_z - view_distance as i32..=chunk_z + view_distance as i32 {
                chunks.push((x, z));
            }
        }
        let protocol_id = player.protocol_version();
        let coords_to_packet = { |coords: (i32, i32)| self.build_chunk_data(coords, protocol_id) };
        player.connection().send_batch(chunks, coords_to_packet);
    }

    pub fn update_player_pos(
        &mut self,
        player: &FalconPlayer,
        old_chunk_x: i32,
        old_chunk_z: i32,
        chunk_x: i32,
        chunk_z: i32,
    ) {
        let view_distance = player.view_distance();
        let render_width = 2 * view_distance as u32 + 1;
        let x_direction = old_chunk_x.abs_diff(chunk_x).max(render_width);
        let z_direction =
            old_chunk_z.abs_diff(chunk_z).max(render_width) * (render_width - x_direction);
        let mut should_load = Vec::with_capacity((x_direction * render_width + z_direction) as usize);
        let mut should_unload = Vec::with_capacity((x_direction * render_width + z_direction) as usize);
        // unload old chunks
        for x in old_chunk_x - view_distance as i32..=old_chunk_x + view_distance as i32 {
            for z in old_chunk_z - view_distance as i32..=old_chunk_z + view_distance as i32 {
                if chunk_x.abs_diff(x) > view_distance as u32
                    || chunk_z.abs_diff(z) > view_distance as u32
                {
                    should_unload.push((x, z));
                }
            }
        }
        // load new chunks
        for x in chunk_x - view_distance as i32..=chunk_x + view_distance as i32 {
            for z in chunk_z - view_distance as i32..=chunk_z + view_distance as i32 {
                if old_chunk_x.abs_diff(x) > view_distance as u32
                    || old_chunk_z.abs_diff(z) > view_distance as u32
                {
                    should_load.push((x, z));
                }
            }
        }
        let protocol_id = player.protocol_version();
        let coords_to_packet = { |coords: (i32, i32)| self.build_chunk_data(coords, protocol_id) };

        player.connection().send_batch(should_load, coords_to_packet);
        player.connection().send_batch(should_unload, |s| falcon_send::build_unload_chunk(s, protocol_id));
    }

    pub fn update_view_distance(&mut self, player: &FalconPlayer, view_distance: u8) {
        let old_view_distance = player.view_distance();
        let (chunk_x, chunk_z) = player.position().chunk_coords();
        let capacity = 4
            * (old_view_distance.abs_diff(view_distance) as usize
                * (old_view_distance + view_distance + 1) as usize);
        match old_view_distance.cmp(&view_distance) {
            std::cmp::Ordering::Less => {
                let mut chunks = Vec::with_capacity(capacity);
                for x in -(view_distance as i8)..=view_distance as i8 {
                    for z in -(view_distance as i8)..=view_distance as i8 {
                        if x.unsigned_abs() > old_view_distance || z.unsigned_abs() > old_view_distance {
                            chunks.push((chunk_x + x as i32, chunk_z + z as i32));
                        }
                    }
                }
                let protocol_id = player.protocol_version();
                let coords_to_packet =
                    { |coords: (i32, i32)| self.build_chunk_data(coords, protocol_id) };

                player.connection().send_batch(chunks, coords_to_packet);
            }
            std::cmp::Ordering::Greater => {
                let mut chunks = Vec::with_capacity(capacity);
                for x in -(old_view_distance as i8)..=old_view_distance as i8 {
                    for z in -(old_view_distance as i8)..=old_view_distance as i8 {
                        if x.unsigned_abs() > view_distance || z.unsigned_abs() > view_distance {
                            chunks.push((chunk_x + x as i32, chunk_z + z as i32));
                        }
                    }
                }
                let protocol_id = player.protocol_version();
                player.connection().send_batch(chunks, |s| falcon_send::build_unload_chunk(s, protocol_id));
            }
            std::cmp::Ordering::Equal => {}
        }
    }
}

impl FalconWorld {
    pub fn build_chunk_data(&mut self, (x, z): (i32, i32), protocol_id: i32) -> Option<Bytes> {
        let mut dirty = false;
        let (non_empty, empty);
        let chunk_data = self.get_chunk((x, z).into());
        let bytes: &dyn Fn() -> Option<Bytes> = match chunk_data {
            Some(chunk_data) => {
                if chunk_data.is_dirty() {
                    dirty = true;
                }
                non_empty = || {
                    falcon_send::build_chunk_data(
                        ChunkDataSpec::new(chunk_data, protocol_id),
                        protocol_id,
                    )
                };
                &non_empty
            }
            None => {
                empty = || falcon_send::build_chunk_data(ChunkDataSpec::empty(x, z), protocol_id);
                &empty
            }
        };
        if dirty {
            let bytes = bytes();
            let packet_cache = self.cache.entry((x, z)).or_default();
            packet_cache.clear();
            packet_cache.insert(protocol_id, bytes.clone());
            self.get_chunk_mut((x, z).into()).mark_dirty(false);
            bytes
        } else if let Some(cache) = self.cache.get(&(x, z)).and_then(|data| data.get(&protocol_id)) {
                cache.clone()
        } else {
            let bytes = bytes();
            self.cache.entry((x, z)).or_default().insert(protocol_id, bytes.clone());
            bytes
        }
    }
}

impl<'a> TryFrom<SchematicData<'a>> for FalconWorld {
    type Error = FalconCoreError;

    #[tracing::instrument(name = "world_loading", skip_all)]
    fn try_from(schematic: SchematicData<'a>) -> Result<Self, Self::Error> {
        let rest_x = schematic.width % 16;
        let rest_z = schematic.length % 16;
        let count_x = ((schematic.width - rest_x) / 16) as usize + if rest_x > 0 { 1 } else { 0 };
        let count_z = ((schematic.length - rest_z) / 16) as usize + if rest_z > 0 { 1 } else { 0 };
        debug!(x = count_x, z = count_z, "World size");

        let air_value = schematic.palette
            .iter()
            .find(|(_, value)| *value == &Blocks::Air)
            .map(|(index, _)| *index);
        let mut schematic_blocks = schematic.block_data
            .iter()
            .map(|b| b as u8)
            .batching(|iter| read_var_i32_from_iter(iter));

        let mut world = FalconWorld::new(count_x * count_z, 0, 0, count_x as i32, count_z as i32);
        for y in 0..schematic.height as usize {
            for z in 0..schematic.length as usize {
                for x in 0..schematic.width as usize {
                    let schematic_block = schematic_blocks.next().ok_or_else(|| FalconCoreError::InvalidData(String::from("Invalid world data, fewer blocks than size given!!")))?;
                    match air_value {
                        Some(value) if value == schematic_block => {}
                        _ => {
                            let chunk_pos = ChunkPos::new(
                                (x / SECTION_WIDTH as usize) as i32,
                                (z / SECTION_LENGTH as usize) as i32,
                            );
                            let palette_entry = *schematic.palette.get(&schematic_block).ok_or_else(|| FalconCoreError::InvalidData(String::from("Invalid schematic data, could not find corresponding palette entry!!")))?;
                            let chunk = world.get_chunk_mut(chunk_pos);
                            chunk.set_block_at(
                                (x as i32 - (chunk_pos.x * SECTION_WIDTH as i32)) as u16,
                                y as u16,
                                (z as i32 - (chunk_pos.z * SECTION_LENGTH as i32)) as u16,
                                palette_entry,
                            );
                        }
                    }
                }
            }
        }
        debug!(count = world.chunks.len(), "Loaded chunks.");
        Ok(world)
    }
}
