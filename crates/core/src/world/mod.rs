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
    width: usize,
    length: usize,
    chunks: AHashMap<ChunkPos, Chunk>,
}

impl World {
    fn new(capacity: usize, width: usize, length: usize) -> Self {
        World {
            width,
            length,
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
    pub fn get_chunks_for_player(&self, _player: &dyn MinecraftPlayer) -> (impl Iterator<Item=&Chunk>, usize, usize) {
        (self.chunks.values(), self.width, self.length)
    }
}

impl TryFrom<SchematicData> for World {
    type Error = Error;

    fn try_from(schematic: SchematicData) -> std::result::Result<Self, Self::Error> {
        let rest_x = schematic.width % 16;
        let rest_z = schematic.length % 16;
        let count_x = ((schematic.width - rest_x) / 16) as usize + 1;
        let count_z = ((schematic.length - rest_z) / 16) as usize + 1;

        let mut world = World::new(count_x * count_z, count_x, count_z);
        for y in 0..schematic.height {
            for z in 0..schematic.length {
                for x in 0..schematic.width {
                    let chunk_pos = ChunkPos::new((x / SECTION_WIDTH) as i32, (z / SECTION_LENGTH) as i32);
                    let chunk = world.get_chunk_mut(chunk_pos);
                    let schematic_block = schematic.block_data[(x + z * schematic.width + y * schematic.width * schematic.length) as usize];
                    chunk.set_block_at((x as i32 - (chunk_pos.x * SECTION_WIDTH as i32)) as u16, y, (z as i32 - (chunk_pos.z * SECTION_LENGTH as i32)) as u16, *schematic.palette.get(&schematic_block).ok_or(Error::from("Invalid schematic data, could not find corresponding palette entry!!"))?);
                }
            }
        }

        debug!("Chunks: {:?}", world.chunks.keys());
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








