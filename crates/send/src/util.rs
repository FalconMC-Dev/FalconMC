use falcon_core::world::block_util::blocks_movement;

use falcon_core::world::blocks::Blocks;
use falcon_core::world::chunks::ChunkSection;
use falcon_core::world::palette::PaletteToI32;
use crate::ChunkSectionDataSpec;

pub struct HeightMap {
    motion_blocking: Vec<u16>,
}

impl HeightMap {
    pub fn from_sections(sections: &[ChunkSectionDataSpec], to_i32: PaletteToI32<Blocks>) -> HeightMap {
        match sections.len() {
            0 => HeightMap {
                motion_blocking: vec![0; 16*16]
            },
            _ => {
                let mut heightmap = Vec::with_capacity(256);
                for x in 0..16 {
                    for z in 0..16 {
                        let mut found = false;
                        for section in sections.iter().rev() {
                            let top_y = section.section_index * 16 + 16;
                            for y in (top_y-16..top_y).rev() {
                                let block = section.palette.at(ChunkSection::calculate_index(x as u16, y as u16, z as u16)).unwrap();
                                if to_i32(block).is_some() && blocks_movement(block) {
                                    heightmap.insert((x + z * 16) as usize, (y + 1) as u16);
                                    found = true;
                                    break;
                                }
                            }
                            if found {
                                break;
                            }
                        }
                    }
                }
                HeightMap {
                    motion_blocking: heightmap,
                }
            }
        }
    }

    pub fn motion_blocking(self) -> Vec<u16> {
        self.motion_blocking
    }
}