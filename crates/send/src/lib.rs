use mc_chat::ChatComponent;
use specs::login::*;
use specs::play::*;
use specs::status::*;

pub mod macros;
pub mod specs;
pub mod util;
pub mod v1_8_9;
pub mod v1_9;
pub mod v1_9_1;
pub mod v1_12_2;
pub mod v1_13;
pub mod v1_14;
pub mod v1_15;
pub mod v1_16;

// Status packets
packet_write_fn! {
    StatusResponseSpec => write_status_response {
        mod v1_8_9::status::status_response;
    }
    i64 => write_status_pong {
        mod v1_8_9::status::status_pong;
    }
}

// Login packets
packet_write_fn! {
    ChatComponent => write_login_disconnect {
        mod v1_8_9::login::disconnect;
    }
    LoginSuccessSpec => write_login_success {
        mod v1_8_9::login::login_success;
        mod v1_16::login::login_success;
    }
}

// Play packets
packet_write_fn! {
    ChatComponent => write_play_disconnect {
        mod v1_8_9::play::disconnect;
    }
    JoinGameSpec => write_join_game {
        mod v1_8_9::play::join_game;
        mod v1_9_1::play::join_game;
        mod v1_14::play::join_game;
        mod v1_15::play::join_game;
        mod v1_16::play::join_game;
    }
    ServerDifficultySpec => write_server_difficulty {
        mod v1_8_9::play::difficulty;
        mod v1_14::play::difficulty;
    }
    PlayerAbilitiesSpec => write_player_abilities {
        mod v1_8_9::play::player_abilities;
    }
    i64 => write_keep_alive {
        mod v1_12_2::play::keep_alive;
    }
    PositionAndLookSpec => write_position_look {
        mod v1_9::play::position_look;
    }
    ChunkDataSpec => write_chunk_data {
        mod v1_13::play::chunk_data;
        mod v1_14::play::chunk_data;
        mod v1_15::play::chunk_data;
        mod v1_16::play::chunk_data;
    }
    (i32, i32) => write_unload_chunk {
        mod v1_9::play::unload_chunk;
    }
    (i32, i32) => write_update_viewpos {
        mod v1_14::play::update_viewpos;
    }
}
