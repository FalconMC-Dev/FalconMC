use falcon_core::network::buffer::{get_var_i32_size, PacketBufferWrite};
use falcon_core::network::packet::PacketEncode;
use falcon_core::player::{GameMode, PlayerAbilityFlags};
use falcon_core::server::Difficulty;
use falcon_core::world::blocks::Blocks;
use falcon_core::world::chunks::{Chunk, ChunkSection, SECTION_HEIGHT, SECTION_LENGTH, SECTION_WIDTH};

#[derive(PacketEncode)]
pub struct JoinGamePacket {
    entity_id: i32,
    game_mode: u8,
    dimension: i32,
    difficulty: u8,
    max_players: u8,
    #[max_length(16)]
    level_type: String,
    reduced_debug: bool,
}

impl JoinGamePacket {
    pub fn new(entity_id: i32, game_mode: GameMode, dimension: i32, difficulty: Difficulty, max_players: u8, level_type: String, reduced_debug: bool) -> Self {
        JoinGamePacket {
            entity_id,
            game_mode: game_mode as u8,
            dimension,
            difficulty: difficulty as u8,
            max_players,
            level_type,
            reduced_debug
        }
    }
}

#[derive(PacketEncode, new)]
pub struct PlayerAbilitiesPacket {
    flags: PlayerAbilityFlags,
    flying_speed: f32,
    fov_modifier: f32,
}

#[derive(PacketEncode, new)]
pub struct PlayerPositionAndLookPacket {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    flags: u8,
    #[var_int]
    teleport_id: i32,
}

pub const MAX_BITS_PER_BLOCK: u8 = 14;

/// For now, we don't support different lighting conditions (meaning all max)
/// and every Biome is the same
pub struct ChunkDataPacket {
    chunk_x: i32,
    chunk_z: i32,
    bit_mask: i32,
    // should determine self
    chunk_sections: Vec<ChunkSectionData>,
}

impl ChunkDataPacket {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let chunk_pos = chunk.get_position();
        let bit_mask = chunk.get_bit_mask();
        let mut chunk_sections = Vec::with_capacity(chunk.get_bit_mask().count_ones() as usize);
        for section in chunk.get_chunk_sections() {
            if let Some(section) = section {
                chunk_sections.push(ChunkSectionData::from_section(section));
            }
        }
        ChunkDataPacket {
            chunk_x: chunk_pos.x,
            chunk_z: chunk_pos.z,
            bit_mask,
            chunk_sections,
        }
    }

    pub fn empty(chunk_x: i32, chunk_z: i32) -> Self {
        ChunkDataPacket {
            chunk_x,
            chunk_z,
            bit_mask: 0,
            chunk_sections: vec![],
        }
    }
}

impl PacketEncode for ChunkDataPacket {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        buf.write_i32(self.chunk_x);
        buf.write_i32(self.chunk_z);
        buf.write_bool(true); // We only send full chunks currently!
        buf.write_var_i32(self.bit_mask);
        let mut data_size: i32 = BIOME_COUNT as i32 * 4; // biomes get sent because of full chunk
        for chunk in &self.chunk_sections {
            data_size += chunk.get_data_size();
        }
        buf.write_var_i32(data_size);
        for chunk in &self.chunk_sections {
            chunk.to_buf(buf);
        }
        for x in BIOMES {
            buf.write_i32(x);
        }
        buf.write_var_i32(0);
    }
}

const BIOME_COUNT: u16 = SECTION_WIDTH * SECTION_LENGTH;
const LIGHT_COUNT: usize = ((SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH) / 2) as usize;
const BIOMES: [i32; BIOME_COUNT as usize] = [0; BIOME_COUNT as usize];
const MAX_LIGHT: [u8; LIGHT_COUNT] = [0xFF; LIGHT_COUNT];

pub struct ChunkSectionData {
    bits_per_block: u8,
    palette: Option<Vec<i32>>,
    block_data: Vec<u64>,
    // light is always the same
}

impl ChunkSectionData {
    pub fn get_data_size(&self) -> i32 {
        let mut size = 1; // always one for bits per block;
        if let Some(palette) = &self.palette {
            size += get_var_i32_size(palette.len() as i32);
            size += palette.iter().map(|x| get_var_i32_size(*x)).sum::<usize>();
        }
        size += get_var_i32_size(self.block_data.len() as i32);
        size += self.block_data.len() * 8;
        size += LIGHT_COUNT;
        size += LIGHT_COUNT; // we only have the overworld for now
        size as i32
    }

    pub fn from_section(chunk_section: &ChunkSection) -> Self {
        let bits_per_block = {
            let actual = usize::BITS - (chunk_section.get_palette().iter()
                .map(|block| block.get_global_id_1631())
                .filter(|o| o.is_some())
                .count() - 1).leading_zeros();
            if actual < 4 {
                4u8
            } else if actual < 9 {
                actual as u8
            } else {
                MAX_BITS_PER_BLOCK
            }
        };

        if bits_per_block > 8 {
            let palette = chunk_section.get_palette();
            const LONG_COUNT: u32 = (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * MAX_BITS_PER_BLOCK as u16) as u32 / i64::BITS;
            let mut block_data = Vec::with_capacity(LONG_COUNT as usize);
            let mut current_long = 0u64;
            let mut offset = 0;
            let mut pos = 0;
            for element in chunk_section.get_block_data().iter().map(|x| palette[*x as usize].get_global_id_1631().unwrap_or(Blocks::Air.get_global_id_1631().unwrap())) {
                let bit_shift = pos * MAX_BITS_PER_BLOCK + offset;
                if bit_shift < (i64::BITS - MAX_BITS_PER_BLOCK as u32) as u8 {
                    current_long |= (element as u64) << bit_shift;
                    pos += 1;
                } else {
                    offset = bit_shift - (i64::BITS - MAX_BITS_PER_BLOCK as u32) as u8;
                    current_long |= (element as u64) << bit_shift;
                    block_data.push(current_long);
                    current_long = 0u64;
                    if offset != 0 {
                        let diff = MAX_BITS_PER_BLOCK - offset;
                        current_long |= (element as u64) >> diff;
                    }
                    pos = 0;
                }
            }

            ChunkSectionData {
                bits_per_block: MAX_BITS_PER_BLOCK,
                palette: None,
                block_data,
            }
        } else {
            let mut palette_missing = 0;
            let mut palette: Vec<i32> = {
                let mut section_palette: Vec<Option<i32>> = chunk_section.get_palette().iter().map(|b| {
                    debug!("Block: {:?}", b);
                    b.get_global_id_1631()
                }).collect();
                let mut i = 0;
                while i < section_palette.len() - palette_missing {
                    if let None = section_palette[i] {
                        section_palette.remove(i);
                        section_palette.push(Some((i + palette_missing) as i32));
                        palette_missing += 1;
                    } else {
                        i += 1;
                    }
                }
                debug!("Palette start!");
                section_palette.iter().map(|b| {
                    debug!("Entry: {}", b.unwrap());
                    b.unwrap()
                }).collect()
            };

            let long_count: u32 = (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16) as u32 / i64::BITS;
            let mut block_data = Vec::with_capacity(long_count as usize);
            let mut current_long = 0u64;
            let mut offset = 0;
            let mut pos = 0;
            for element in chunk_section.get_block_data().iter().map(|x| {
                let palette_len = palette.len();
                if palette[palette_len-palette_missing..palette_len].contains(&(*x as i32)) {
                    0
                } else {
                    let mut res = *x;
                    for j in &palette[palette_len-palette_missing..palette_len] {
                        if *x > *j as u16 {
                            res -= 1
                        }
                    }
                    res
                }
            }) {
                let bit_shift = pos * bits_per_block + offset;
                if bit_shift < (i64::BITS - bits_per_block as u32) as u8 {
                    current_long |= (element as u64) << bit_shift;
                    pos += 1;
                } else {
                    offset = bit_shift - (i64::BITS - bits_per_block as u32) as u8;
                    current_long |= (element as u64) << bit_shift;
                    block_data.push(current_long);
                    current_long = 0u64;
                    if offset != 0 {
                        let diff = bits_per_block - offset;
                        current_long |= (element as u64) >> diff;
                    }
                    pos = 0;
                }
            }
            palette.drain(palette.len() - palette_missing..palette.len());

            ChunkSectionData {
                bits_per_block,
                palette: Some(palette),
                block_data
            }
        }
    }
}

impl PacketEncode for ChunkSectionData {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        buf.write_u8(self.bits_per_block);
        if let Some(palette) = &self.palette {
            buf.write_var_i32(palette.len() as i32);
            for x in palette {
                buf.write_var_i32(*x);
            }
        }
        buf.write_var_i32(self.block_data.len() as i32);
        for x in &self.block_data {
            buf.write_i64(*x as i64);
        }
        for x in MAX_LIGHT {
            buf.write_u8(x);
        }
        for x in MAX_LIGHT {
            buf.write_u8(x);
        }
    }
}