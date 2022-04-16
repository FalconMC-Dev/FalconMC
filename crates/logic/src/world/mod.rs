use falcon_core::player::Player;
use falcon_core::world::chunks::ChunkPos;
use falcon_core::world::World;
use falcon_send::specs::play::ChunkDataSpec;

/// Initialize terrain when player spawns
pub fn send_chunks_for_player(world: &World, player: &Player) {
    let (chunk_x, chunk_z) = player.position().chunk_coords();
    let view_distance = player.view_distance();
    let capacity = (2 * view_distance as usize + 1).pow(2);
    let mut chunks = Vec::with_capacity(capacity);

    for x in chunk_x - view_distance as i32..=chunk_x + view_distance as i32 {
        for z in chunk_z - view_distance as i32..=chunk_z + view_distance as i32 {
            match world.get_chunk(ChunkPos::new(x, z)) {
                None => chunks.push(ChunkDataSpec::empty(x, z)),
                Some(chunk) => chunks.push(ChunkDataSpec::new(chunk, player.protocol_version())),
            }
        }
    }
    falcon_send::batch::send_batch(chunks, |s| s, falcon_send::build_chunk_data, player.protocol_version(), player.connection());
}

pub fn update_player_pos(
    world: &World,
    player: &Player,
    old_chunk_x: i32,
    old_chunk_z: i32,
    chunk_x: i32,
    chunk_z: i32
) {
    let view_distance = player.view_distance();
    let render_width = 2 * view_distance as u32 + 1;
    let x_direction = old_chunk_x.abs_diff(chunk_x).max(render_width);
    let z_direction = old_chunk_z.abs_diff(chunk_z).max(render_width) * (render_width - x_direction);
    let mut should_load = Vec::with_capacity((x_direction * render_width + z_direction) as usize);
    let mut should_unload = Vec::with_capacity((x_direction * render_width + z_direction) as usize);
    // unload old chunks
    for x in old_chunk_x - view_distance as i32..=old_chunk_x + view_distance as i32 {
        for z in old_chunk_z - view_distance as i32..=old_chunk_z + view_distance as i32 {
            if chunk_x.abs_diff(x) > view_distance as u32 || chunk_z.abs_diff(z) > view_distance as u32 {
                should_unload.push((x, z));
            }
        }
    }
    // load new chunks
    for x in chunk_x - view_distance as i32..=chunk_x + view_distance as i32 {
        for z in chunk_z - view_distance as i32..=chunk_z + view_distance as i32 {
            if old_chunk_x.abs_diff(x) > view_distance as u32 || old_chunk_z.abs_diff(z) > view_distance as u32 {
                match world.get_chunk(ChunkPos::new(x, z)) {
                    None => should_load.push(ChunkDataSpec::empty(x, z)),
                    Some(chunk) => should_load.push(ChunkDataSpec::new(chunk, player.protocol_version())),
                }
            }
        }
    }
    falcon_send::batch::send_batch(should_load, |s| s, falcon_send::build_chunk_data, player.protocol_version(), player.connection());
    falcon_send::batch::send_batch(should_unload, |s| s, falcon_send::build_unload_chunk, player.protocol_version(), player.connection());
}

#[allow(clippy::comparison_chain)]
pub fn update_view_distance(world: &World, player: &Player, view_distance: u8) {
    let old_view_distance = player.view_distance();
    let (chunk_x, chunk_z) = player.position().chunk_coords();
    let capacity = 4 * (old_view_distance.abs_diff(view_distance) as usize * (old_view_distance + view_distance + 1) as usize);
    if old_view_distance < view_distance {
        let mut chunks = Vec::with_capacity(capacity);
        for x in -(view_distance as i8)..=view_distance as i8 {
            for z in -(view_distance as i8)..=view_distance as i8 {
                if x.abs() as u8 > old_view_distance || z.abs() as u8 > old_view_distance {
                    match world.get_chunk(ChunkPos::new(chunk_x + x as i32, chunk_z + z as i32)) {
                        None => chunks.push(ChunkDataSpec::empty(chunk_x + x as i32, chunk_z + z as i32)),
                        Some(chunk) => chunks.push(ChunkDataSpec::new(chunk, player.protocol_version())),
                    }
                }
            }
        }
        falcon_send::batch::send_batch(chunks, |s| s, falcon_send::build_chunk_data, player.protocol_version(), player.connection());
    } else if old_view_distance > view_distance {
        let mut chunks = Vec::with_capacity(capacity);
        for x in -(old_view_distance as i8)..=old_view_distance as i8 {
            for z in -(old_view_distance as i8)..=old_view_distance as i8 {
                if x.abs() as u8 > view_distance || z.abs() as u8 > view_distance {
                    chunks.push((chunk_x + x as i32, chunk_z + z as i32));
                }
            }
        }
        falcon_send::batch::send_batch(chunks, |s| s, falcon_send::build_unload_chunk, player.protocol_version(), player.connection());
    }
}