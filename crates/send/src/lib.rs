#[macro_use]
extern crate tracing;

use specs::status::*;
use specs::login::*;
use specs::play::*;
use mc_chat::ChatComponent;

pub mod batch;
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
packet_send_fn! {
    StatusResponseSpec => send_status_response {
        mod v1_8_9::status::status_response;
    }
    i64 => send_status_pong {
        mod v1_8_9::status::status_pong;
    }
}

// Login packets
packet_send_fn! {
    ChatComponent => send_login_disconnect {
        mod v1_8_9::login::disconnect;
    }
    LoginSuccessSpec => send_login_success {
        mod v1_8_9::login::login_success;
        mod v1_16::login::login_success;
    }
}

// Play packets
packet_send_fn! {
    ChatComponent => send_play_disconnect {
        mod v1_8_9::play::disconnect;
    }
    JoinGameSpec => send_join_game {
        mod v1_8_9::play::join_game;
        mod v1_9_1::play::join_game;
        mod v1_14::play::join_game;
        mod v1_15::play::join_game;
        mod v1_16::play::join_game;
    }
    ServerDifficultySpec => send_server_difficulty {
        mod v1_8_9::play::send_difficulty;
        mod v1_14::play::send_difficulty;
    }
    PlayerAbilitiesSpec => send_player_abilities {
        mod v1_8_9::play::player_abilities;
    }
    i64 => send_keep_alive {
        mod v1_12_2::play::keep_alive;
    }
    PositionAndLookSpec => send_position_look {
        mod v1_9::play::position_look;
    }
    ChunkDataSpec => send_chunk_data {
        mod v1_13::play::chunk_data;
        mod v1_14::play::chunk_data;
        mod v1_15::play::chunk_data;
        mod v1_16::play::chunk_data;
    }
    (i32, i32) => send_unload_chunk {
        mod v1_9::play::unload_chunk;
    }
    (i32, i32) => send_update_viewpos {
        mod v1_14::play::update_viewpos;
    }
}

build_send_fn! {
    ChunkDataSpec => build_chunk_data {
        mod v1_13::play::build_chunk_data;
        mod v1_14::play::build_chunk_data;
        mod v1_15::play::build_chunk_data;
        mod v1_16::play::build_chunk_data;
    }
    (i32, i32) => build_unload_chunk {
        mod v1_9::play::build_unload_chunk;
    }
}
