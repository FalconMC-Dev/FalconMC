use crate::player::FalconPlayer;
use crate::world::cache::WorldPacketCache;
use ahash::AHashMap;
use bytes::Bytes;
use falcon_core::network::connection::ConnectionDriver;
use falcon_core::world::chunks::{ChunkPos, Chunk};
use falcon_send::specs::play::ChunkDataSpec;

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

    pub fn send_chunks_for_player<D: ConnectionDriver>(&mut self, player: &FalconPlayer<D>) {
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
        let coords_to_packet = {
            |coords: (i32, i32)| {
                self.build_chunk_data(coords, protocol_id)
            }
        };
        falcon_send::batch::send_batch(chunks, coords_to_packet, player.connection());
    }

    pub fn update_player_pos<D: ConnectionDriver>(
        &mut self,
        player: &FalconPlayer<D>,
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
        let coords_to_packet = {
            |coords: (i32, i32)| {
                self.build_chunk_data(coords, protocol_id)
            }
        };

        falcon_send::batch::send_batch(should_load, coords_to_packet, player.connection());
        falcon_send::batch::send_batch(
            should_unload,
            |s| falcon_send::build_unload_chunk(s, protocol_id),
            player.connection(),
        );
    }

    pub fn update_view_distance<D: ConnectionDriver>(&mut self, player: &FalconPlayer<D>, view_distance: u8) {
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
                let coords_to_packet = {
                    |coords: (i32, i32)| {
                        self.build_chunk_data(coords, protocol_id)
                    }
                };

                falcon_send::batch::send_batch(chunks, coords_to_packet, player.connection());
            },
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
                falcon_send::batch::send_batch(
                    chunks,
                    |s| falcon_send::build_unload_chunk(s, protocol_id),
                    player.connection(),
                );
            },
            std::cmp::Ordering::Equal => {},
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
                non_empty = || falcon_send::build_chunk_data(ChunkDataSpec::new(chunk_data, protocol_id), protocol_id);
                &non_empty
            },
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

