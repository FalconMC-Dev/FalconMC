use std::ops::{Deref, DerefMut};

use ahash::AHashMap;
use bytes::Bytes;

#[derive(Debug, Default)]
pub struct WorldPacketCache {
    packet_data: AHashMap<(i32, i32), PacketCacheData>,
}

impl Deref for WorldPacketCache {
    type Target = AHashMap<(i32, i32), PacketCacheData>;

    fn deref(&self) -> &Self::Target {
        &self.packet_data
    }
}

impl DerefMut for WorldPacketCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.packet_data
    }
}

impl WorldPacketCache {
    /// TODO: do not forget to clear the cache later in dev when chunks get unloaded from the disk (terrain generation extensibility for example)
    pub fn clear(&mut self, coords: (i32, i32)) {
        self.packet_data.remove(&coords);
    }
}

#[derive(Debug, Default)]
pub struct PacketCacheData {
    version_data: AHashMap<i32, Option<Bytes>>,
}

impl Deref for PacketCacheData {
    type Target = AHashMap<i32, Option<Bytes>>;

    fn deref(&self) -> &Self::Target {
        &self.version_data
    }
}

impl DerefMut for PacketCacheData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.version_data
    }
}

impl PacketCacheData {
    pub fn clear(&mut self) {
        self.version_data.clear()
    }
}
