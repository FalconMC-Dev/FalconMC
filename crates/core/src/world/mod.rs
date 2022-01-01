use crate::errors::*;

use ahash::AHashMap;
use falcon_core::player::MinecraftPlayer;
use falcon_core::world::chunks::SECTION_LENGTH;
use crate::player::Position;
use crate::schematic::SchematicData;
use crate::world::chunks::{Chunk, ChunkPos, SECTION_WIDTH};

pub mod chunks;
pub mod blocks;

#[derive(Debug)]
pub struct World {
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
    chunks: AHashMap<ChunkPos, Chunk>,
}

impl World {
    fn new(capacity: usize, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> Self {
        World {
            min_x,
            min_z,
            max_x,
            max_z,
            chunks: AHashMap::with_capacity(capacity),
        }
    }

    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    fn get_chunk_or_create(&mut self, pos: ChunkPos) -> &Chunk {
        if self.chunks.contains_key(&pos) {
            self.chunks.get(&pos).expect(&format!("Hashmap just said there was a chunk for this key {:?}", pos))
        } else {
            let chunk = Chunk::empty(pos);
            self.chunks.insert(pos, chunk);
            self.chunks.get(&pos).expect("We should always get the chunk back we just put in the map")
        }
    }

    fn get_chunk_mut(&mut self, pos: ChunkPos) -> &mut Chunk {
        if self.chunks.contains_key(&pos) {
            self.chunks.get_mut(&pos).expect(&format!("Hashmap just said there was a chunk for this key {:?}", pos))
        } else {
            let chunk = Chunk::empty(pos);
            self.chunks.insert(pos, chunk);
            self.chunks.get_mut(&pos).expect("We should always get the chunk back we just put in the map")
        }
    }

    /// Currently it just sends all chunks known to the world
    /// for a more sophisticated implementation this could filter for chunks centered around the player
    pub fn send_chunks_for_player<C, A>(&self, player: &mut dyn MinecraftPlayer, chunk_fn: C, air_fn: A) -> Result<()>
        where C: Fn(&mut dyn MinecraftPlayer, &Chunk) -> Result<()>,
              A: Fn(&mut dyn MinecraftPlayer, i32, i32) -> Result<()>,
    {
        for chunk in self.chunks.values() {
            chunk_fn(player, chunk)?;
        }
        for i in self.min_x-1..=self.max_x {
            for j in self.min_z-1..=self.max_z {
                if !((i >= self.min_x && j >= self.min_z) && (i < self.max_x && j < self.max_z)) {
                    air_fn(player, i, j)?;
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<SchematicData> for World {
    type Error = Error;

    #[tracing::instrument(name = "world_loading", skip_all)]
    fn try_from(schematic: SchematicData) -> std::result::Result<Self, Self::Error> {
        let rest_x = schematic.width % 16;
        let rest_z = schematic.length % 16;
        let count_x = ((schematic.width - rest_x) / 16) as usize + if rest_x > 0 { 1 } else { 0 };
        let count_z = ((schematic.length - rest_z) / 16) as usize + if rest_z > 0 { 1 } else { 0 };

        let mut world = World::new(count_x * count_z, 0, 0, count_x as i32, count_z as i32);
        for y in 0..schematic.height as usize {
            for z in 0..schematic.length as usize {
                for x in 0..schematic.width as usize {
                    let chunk_pos = ChunkPos::new((x / SECTION_WIDTH as usize) as i32, (z / SECTION_LENGTH as usize) as i32);
                    let chunk = world.get_chunk_mut(chunk_pos);
                    let schematic_block = schematic.block_data[x + z * schematic.width as usize + y * schematic.width as usize * schematic.length as usize];
                    chunk.set_block_at((x as i32 - (chunk_pos.x * SECTION_WIDTH as i32)) as u16, y as u16, (z as i32 - (chunk_pos.z * SECTION_LENGTH as i32)) as u16, *schematic.palette.get(&schematic_block).ok_or(Error::from("Invalid schematic data, could not find corresponding palette entry!!"))?);
                }
            }
        }
        debug!(count = world.chunks.len(), "Loaded chunks.");
        Ok(world)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BlockPosition {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        BlockPosition {
            x,
            y,
            z
        }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn set_z(&mut self, z: i32) {
        self.z = z;
    }
}

impl From<Position> for BlockPosition {
    fn from(pos: Position) -> Self {
        BlockPosition::new(pos.get_x().floor() as i32, pos.get_y().floor() as i32, pos.get_z().floor() as i32)
    }
}








