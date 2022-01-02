use crate::world::blocks::Blocks;

pub const SECTIONS_NUM: u16 = 16;
pub const SECTION_WIDTH: u16 = 16;
pub const SECTION_LENGTH: u16 = 16;
pub const SECTION_HEIGHT: u16 = 16;

#[derive(Debug)]
pub struct Chunk {
    sections: [Option<ChunkSection>; SECTIONS_NUM as usize],
    bitmask: i32,
    pos: ChunkPos,
}

impl Chunk {
    pub fn empty(pos: ChunkPos) -> Self {
        Chunk {
            sections: Default::default(),
            bitmask: 0,
            pos,
        }
    }

    pub fn set_block_at(&mut self, x: u16, y: u16, z: u16, block_state: Blocks) {
        let section_y = y / SECTION_HEIGHT;
        if let Some(section) = &mut self.sections[section_y as usize] {
            section.set_block_at(x, y - (section_y * SECTION_HEIGHT), z, block_state);
            if section.block_count == 0 {
                self.sections[section_y as usize] = None;
                self.bitmask ^= 1 << section_y;
            }
        } else if block_state != Blocks::Air {
            let mut section = ChunkSection::empty();
            section.set_block_at(x, y - (section_y * SECTION_HEIGHT), z, block_state);
            self.sections[section_y as usize] = Some(section);
            self.bitmask ^= 1 << section_y;
        }
    }

    pub fn get_bit_mask(&self) -> i32 {
        self.bitmask
    }

    pub fn get_position(&self) -> &ChunkPos {
        &self.pos
    }

    pub fn get_chunk_sections(&self) -> &[Option<ChunkSection>; SECTIONS_NUM as usize] {
        &self.sections
    }
}

#[derive(Clone, Debug)]
pub struct ChunkSection {
    block_count: u16,
    palette: Vec<Blocks>,
    blocks: Vec<u16>,
}

impl ChunkSection {
    pub fn empty() -> Self {
        ChunkSection {
            block_count: 0,
            palette: vec![Blocks::Air],
            blocks: vec![0u16; (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH) as usize],
        }
    }

    pub fn set_block_at(&mut self, x: u16, y: u16, z: u16, block_state: Blocks) {
        let old_value = self.blocks[Self::calculate_index(x, y, z)];
        if block_state == Blocks::Air && old_value != 0 {
            self.block_count -= 1;
            if !self.blocks.contains(&old_value) {
                self.palette.swap_remove(old_value as usize);
                for value in self.blocks.iter_mut() {
                    if *value > old_value {
                        *value -= 1;
                    }
                }
            }
        } else if block_state != Blocks::Air && old_value == 0 {
            self.block_count += 1;
        }
        if let Some(index) = self.palette.iter().enumerate().find(|(_, block)| *block == &block_state).map(|(i, _)| i) {
            self.blocks[(x + z * SECTION_WIDTH + y * SECTION_WIDTH * SECTION_LENGTH) as usize] = index as u16;
        } else {
            self.palette.push(block_state);
            self.blocks[(x + z * SECTION_WIDTH + y * SECTION_WIDTH * SECTION_LENGTH) as usize] = (self.palette.len() - 1) as u16;
        }
    }

    pub fn get_block_count(&self) -> u16 {
        self.block_count
    }

    pub fn get_palette(&self) -> &Vec<Blocks> {
        &self.palette
    }

    pub fn get_block_data(&self) -> &Vec<u16> {
        &self.blocks
    }

    fn calculate_index(x: u16, y: u16, z: u16) -> usize {
        (x + z * SECTION_WIDTH + y * SECTION_WIDTH * SECTION_LENGTH) as usize
    }
}

impl Default for ChunkSection {
    fn default() -> Self {
        ChunkSection::empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        ChunkPos {
            x,
            z
        }
    }
}