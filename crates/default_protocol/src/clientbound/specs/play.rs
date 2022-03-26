use falcon_core::player::{GameMode, MinecraftPlayer, PlayerAbilityFlags};
use falcon_core::server::Difficulty;
use falcon_core::world::blocks::Blocks;
use falcon_core::world::chunks::{Chunk, ChunkSection};
use falcon_core::world::palette::Palette;
use crate::define_spec;

define_spec! {
    JoinGameSpec => player: &dyn MinecraftPlayer {
        difficulty: Difficulty,
        max_players: u8,
        level_type: String,
        view_distance: i32,
        reduced_debug: bool;
        let entity_id: i32 = player.entity_id(),
        let game_mode: GameMode = player.game_mode(),
        let dimension: i32 = player.dimension()
    }
}

define_spec! {
    PlayerAbilitiesSpec => player: &dyn MinecraftPlayer {
        flying_speed: f32,
        fov_modifier: f32;
        let flags: PlayerAbilityFlags = player.ability_flags(),
    }
}

define_spec! {
    PositionAndLookSpec => player: &dyn MinecraftPlayer {
        flags: u8,
        teleport_id: i32;
        let x: f64 = pos.x(),
        let y: f64 = pos.y(),
        let z: f64 = pos.z(),
        let yaw: f32 = look.yaw(),
        let pitch: f32 = look.pitch();
        {
            let pos = *player.position()
            let look = *player.look_angles()
        }
    }
}

define_spec! {
    ChunkDataSpec => chunk: &Chunk, protocol_version: i32 {
        ;let chunk_x: i32 = chunk_pos.x,
        let chunk_z: i32 = chunk_pos.z,
        let bitmask: i32 = bit_mask,
        let sections: Vec<ChunkSectionDataSpec> = chunk_sections;
        {
            let chunk_pos = chunk.get_position()
            let bit_mask = chunk.get_bit_mask()
            let mut chunk_sections = Vec::with_capacity(bit_mask.count_ones() as usize)
            for section in chunk.get_chunk_sections().iter().flatten() {
                chunk_sections.push(ChunkSectionDataSpec::new(section, protocol_version));
            }
        }
    }
}

impl ChunkDataSpec {
    pub fn empty(x: i32, z: i32) -> Self {
        ChunkDataSpec {
            chunk_x: x,
            chunk_z: z,
            bitmask: 0,
            sections: vec![]
        }
    }
}

define_spec! {
    ChunkSectionDataSpec => section: &ChunkSection {
        protocol_version: i32;
        let palette: Palette<Blocks> = section.get_palette().clone(),
        let blocks: Vec<u16> = section.get_block_data().clone(),
    }
}