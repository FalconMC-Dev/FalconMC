use crate::packet_send_fn;

use specs::status::*;
use specs::login::*;
use specs::play::*;
use mc_chat::ChatComponent;

pub mod v1_8_9;
pub mod v1_9;
pub mod v1_9_1;
pub mod v1_12_2;
pub mod v1_13;
pub mod specs;

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
    }
    (i32, i32) => send_unload_chunk {
        mod v1_9::play::unload_chunk;
    }
}