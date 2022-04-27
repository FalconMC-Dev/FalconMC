use ahash::AHashMap;
use bytes::Bytes;
use falcon_core::world::World;
use falcon_send::specs::play::ChunkDataSpec;

#[derive(Debug, Default)]
pub struct WorldPacketCache {
    packet_data: AHashMap<(i32, i32), PacketCacheData>,
}

impl WorldPacketCache {
    pub fn build_chunk_data(&mut self, world: &mut World, (x, z): (i32, i32), protocol_id: i32) -> Option<Bytes> {
        let chunk_cache = self.packet_data.entry((x, z)).or_default();
        let chunk_data = world.get_chunk((x, z).into());
        let mut dirty = false;
        let bytes = match chunk_data {
            Some(chunk_data) => {
                if chunk_data.is_dirty() {
                    dirty = true;
                    chunk_cache.clear();
                }
                chunk_cache.version_data.entry(protocol_id).or_insert_with(|| {
                    falcon_send::build_chunk_data(ChunkDataSpec::new(chunk_data, protocol_id), protocol_id)
                }).clone()
            }
            None => {
                chunk_cache.version_data.entry(protocol_id).or_insert_with(|| {
                    falcon_send::build_chunk_data(ChunkDataSpec::empty(x, z), protocol_id)
                }).clone()
            }
        };
        if dirty {
            world.get_chunk_mut((x, z).into()).mark_dirty(false);
        }
        bytes
    }

    /// TODO: do not forget to clear the cache later in dev when chunks get unloaded from the disk (terrain generation extensibility for example)
    pub fn clear(&mut self, coords: (i32, i32)) {
        self.packet_data.remove(&coords);
    }
}

#[derive(Debug, Default)]
struct PacketCacheData {
    version_data: AHashMap<i32, Option<Bytes>>,
}

impl PacketCacheData {
    pub fn clear(&mut self) {
        self.version_data.clear()
    }
}
