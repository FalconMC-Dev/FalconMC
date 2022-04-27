use once_cell::sync::Lazy;
use parking_lot::Mutex;
use falcon_core::player::Player;
use falcon_core::world::World;
use crate::world::cache::WorldPacketCache;

mod cache;

static PACKET_CACHE: Lazy<Mutex<WorldPacketCache>> = Lazy::new(|| Mutex::new(Default::default()));

/// Initialize terrain when player spawns
pub fn send_chunks_for_player(world: &mut World, player: &Player) {
    let (chunk_x, chunk_z) = player.position().chunk_coords();
    let view_distance = player.view_distance();
    let capacity = (2 * view_distance as usize + 1).pow(2);
    let mut chunks = Vec::with_capacity(capacity);

    for x in chunk_x - view_distance as i32..=chunk_x + view_distance as i32 {
        for z in chunk_z - view_distance as i32..=chunk_z + view_distance as i32 {
            chunks.push((x, z));
        }
    }
    let protocol_id = player.protocol_version();
    let coords_to_packet = {
        |coords: (i32, i32)| {
            let mut cache = PACKET_CACHE.lock();
            cache.build_chunk_data(world, coords, protocol_id)
        }
    };
    falcon_send::batch::send_batch(chunks, coords_to_packet, player.connection());
}

pub fn update_player_pos(
    world: &mut World,
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
                should_load.push((x, z));
            }
        }
    }
    let protocol_id = player.protocol_version();
    let coords_to_packet = {
        |coords: (i32, i32)| {
            let mut cache = PACKET_CACHE.lock();
            cache.build_chunk_data(world, coords, protocol_id)
        }
    };
    falcon_send::batch::send_batch(should_load, coords_to_packet, player.connection());
    falcon_send::batch::send_batch(should_unload, |s| falcon_send::build_unload_chunk(s, protocol_id), player.connection());
}

#[allow(clippy::comparison_chain)]
pub fn update_view_distance(world: &mut World, player: &Player, view_distance: u8) {
    let old_view_distance = player.view_distance();
    let (chunk_x, chunk_z) = player.position().chunk_coords();
    let capacity = 4 * (old_view_distance.abs_diff(view_distance) as usize * (old_view_distance + view_distance + 1) as usize);
    if old_view_distance < view_distance {
        let mut chunks = Vec::with_capacity(capacity);
        for x in -(view_distance as i8)..=view_distance as i8 {
            for z in -(view_distance as i8)..=view_distance as i8 {
                if x.abs() as u8 > old_view_distance || z.abs() as u8 > old_view_distance {
                    chunks.push((chunk_x + x as i32, chunk_z + z as i32));
                }
            }
        }
        let protocol_id = player.protocol_version();
        let coords_to_packet = {
            |coords: (i32, i32)| {
                let mut cache = PACKET_CACHE.lock();
                cache.build_chunk_data(world, coords, protocol_id)
            }
        };
        falcon_send::batch::send_batch(chunks, coords_to_packet, player.connection());
    } else if old_view_distance > view_distance {
        let mut chunks = Vec::with_capacity(capacity);
        for x in -(old_view_distance as i8)..=old_view_distance as i8 {
            for z in -(old_view_distance as i8)..=old_view_distance as i8 {
                if x.abs() as u8 > view_distance || z.abs() as u8 > view_distance {
                    chunks.push((chunk_x + x as i32, chunk_z + z as i32));
                }
            }
        }
        let protocol_id = player.protocol_version();
        falcon_send::batch::send_batch(chunks, |s| falcon_send::build_unload_chunk(s, protocol_id), player.connection());
    }
}