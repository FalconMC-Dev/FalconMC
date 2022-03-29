use crate::error::FalconCoreError;

use crate::network::buffer::read_var_i32_from_iter;
use crate::player::Player;
use crate::player::data::Position;
use crate::schematic::SchematicData;
use crate::world::blocks::Blocks;
use crate::world::chunks::{Chunk, ChunkPos, SECTION_WIDTH};
use ahash::AHashMap;
use falcon_core::world::chunks::SECTION_LENGTH;
use itertools::Itertools;

pub mod blocks;
pub mod chunks;
pub mod palette;

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
        if pos.x > self.max_x || pos.x < self.min_x || pos.z > self.max_z || pos.z < self.min_z {
            return None;
        }
        self.chunks.get(&pos)
    }

    fn get_chunk_mut(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.chunks.entry(pos).or_insert_with(|| Chunk::empty(pos))
    }

    /// Initialize terrain when player spawns
    pub fn send_chunks_for_player<C, A>(
        &self,
        player: &mut Player,
        chunk_fn: C,
        air_fn: A,
    )
    where
        C: Fn(&mut Player, &Chunk),
        A: Fn(&mut Player, i32, i32),
    {
        let (chunk_x, chunk_z) = player.position().chunk_coords();
        let view_distance = player.view_distance();

        for x in chunk_x - view_distance as i32..=chunk_x + view_distance as i32 {
            for z in chunk_z - view_distance as i32..=chunk_z + view_distance as i32 {
                match self.get_chunk(ChunkPos::new(x, z)) {
                    None => air_fn(player, x, z),
                    Some(chunk) => chunk_fn(player, chunk),
                }
            }
        }
    }

    pub fn update_player_pos<C, A, U>(
        &self,
        player: &mut Player,
        old_chunk_x: i32,
        old_chunk_z: i32,
        chunk_x: i32,
        chunk_z: i32,
        chunk_fn: C,
        air_fn: A,
        unload_fn: U,
    )
    where
        C: Fn(&mut Player, &Chunk),
        A: Fn(&mut Player, i32, i32),
        U: Fn(&mut Player, i32, i32),
    {
        let view_distance = player.view_distance();
        // unload old chunks
        for x in old_chunk_x - view_distance as i32..=old_chunk_x + view_distance as i32 {
            for z in old_chunk_z - view_distance as i32..=old_chunk_z + view_distance as i32 {
                if (chunk_x - x).abs() > view_distance as i32 || (chunk_z - z).abs() > view_distance as i32 {
                    unload_fn(player, x, z);
                }
            }
        }
        // load new chunks
        for x in chunk_x - view_distance as i32..=chunk_x + view_distance as i32 {
            for z in chunk_z - view_distance as i32..=chunk_z + view_distance as i32 {
                if (old_chunk_x - x).abs() > view_distance as i32 || (old_chunk_z - z).abs() > view_distance as i32 {
                    match self.get_chunk(ChunkPos::new(x, z)) {
                        None => air_fn(player, x, z),
                        Some(chunk) => chunk_fn(player, chunk),
                    }
                }
            }
        }
    }

    #[allow(clippy::comparison_chain)]
    pub fn update_view_distance<C, A, U>(
        &self,
        player: &mut Player,
        view_distance: u8,
        chunk_fn: C,
        air_fn: A,
        unload_fn: U
    )
    where
        C: Fn(&mut Player, &Chunk),
        A: Fn(&mut Player, i32, i32),
        U: Fn(&mut Player, i32, i32),
    {
        let old_view_distance = player.view_distance();
        let (chunk_x, chunk_z) = player.position().chunk_coords();
        if old_view_distance < view_distance {
            for x in -(view_distance as i8)..=view_distance as i8 {
                for z in -(view_distance as i8)..=view_distance as i8 {
                    if x.abs() as u8 > old_view_distance || z.abs() as u8 > old_view_distance {
                        match self.get_chunk(ChunkPos::new(chunk_x + x as i32, chunk_z + z as i32)) {
                            None => air_fn(player, chunk_x + x as i32, chunk_z + z as i32),
                            Some(chunk) => chunk_fn(player, chunk),
                        }
                    }
                }
            }
        } else if old_view_distance > view_distance {
            for x in -(old_view_distance as i8)..=old_view_distance as i8 {
                for z in -(old_view_distance as i8)..=old_view_distance as i8 {
                    if x.abs() as u8 > view_distance || z.abs() as u8 > view_distance {
                        unload_fn(player, chunk_x + x as i32, chunk_z + z as i32);
                    }
                }
            }
        }
    }
}

impl<'a> TryFrom<SchematicData<'a>> for World {
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

        let mut world = World::new(count_x * count_z, 0, 0, count_x as i32, count_z as i32);
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
                            chunk.set_block_at((x as i32 - (chunk_pos.x * SECTION_WIDTH as i32)) as u16, y as u16, (z as i32 - (chunk_pos.z * SECTION_LENGTH as i32)) as u16, palette_entry);
                        }
                    }
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
        BlockPosition { x, y, z }
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
        BlockPosition::new(
            pos.x().floor() as i32,
            pos.y().floor() as i32,
            pos.z().floor() as i32,
        )
    }
}
