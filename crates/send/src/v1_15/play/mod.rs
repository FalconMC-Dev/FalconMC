mod chunk;

pub use chunk::*;
#[falcon_send_derive::falcon_send]
mod inner {
    use derive_from_ext::From;
    use falcon_packet_core::{PacketSize, PacketWrite};

    use crate::JoinGameSpec;

    #[derive(PacketSize, PacketWrite, From)]
    #[from(JoinGameSpec)]
    #[falcon_packet(versions = {
        573, 575, 578 = 0x26;
    }, name = "join_game")]
    pub struct JoinGamePacket {
        entity_id: i32,
        game_mode: u8,
        dimension: i32,
        hashed_seed: i64,
        max_players: u8,
        #[falcon(string = 16)]
        level_type: String,
        #[falcon(var32)]
        view_distance: i32,
        reduced_debug: bool,
        enable_respawn_screen: bool,
    }
}
