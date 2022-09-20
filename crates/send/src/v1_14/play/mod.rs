mod chunk;

pub use chunk::*;

#[falcon_send_derive::falcon_send]
mod inner {
    use crate::{JoinGameSpec, ServerDifficultySpec};
    use derive_from_ext::From;
    use falcon_packet_core::{PacketSize, PacketWrite};

    #[derive(PacketSize, PacketWrite, From)]
    #[from(JoinGameSpec)]
    #[falcon_packet(versions = {
        477, 480, 485, 490, 498 = 0x25;
    }, name = "join_game")]
    pub struct JoinGamePacket {
        entity_id: i32,
        game_mode: u8,
        dimension: i32,
        max_players: u8,
        #[falcon(string = 16)]
        level_type: String,
        #[falcon(var32)]
        view_distance: i32,
        reduced_debug: bool,
    }

    #[derive(PacketSize, PacketWrite, From)]
    #[from(ServerDifficultySpec)]
    #[falcon_packet(versions = {
        477, 480, 485, 490, 498, 735, 736 = 0x0D;
        573, 575, 578 = 0x0E;
    }, name = "send_difficulty")]
    pub struct ServerDifficultyPacket {
        difficulty: u8,
        locked: bool,
    }

    #[derive(PacketSize, PacketWrite)]
    #[falcon_packet(versions = {
        477, 480, 485, 490, 498, 735, 736 = 0x40;
        573, 575, 578 = 0x41;
    }, name = "update_viewpos")]
    pub struct UpdateViewPosition {
        #[falcon(var32)]
        chunk_x: i32,
        #[falcon(var32)]
        chunk_z: i32,
    }

    impl From<(i32, i32)> for UpdateViewPosition {
        fn from((chunk_x, chunk_z): (i32, i32)) -> Self {
            UpdateViewPosition { chunk_x, chunk_z }
        }
    }
}
