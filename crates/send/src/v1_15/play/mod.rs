pub use inner::*;
pub use chunk::*;

mod chunk;

#[falcon_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::packet::PacketEncode;
    use crate::JoinGameSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(573, 575, 578 = 0x26; no_receive; outgoing = "join_game")]
    pub struct JoinGamePacket {
        entity_id: i32,
        game_mode: u8,
        dimension: i32,
        hashed_seed: i64,
        max_players: u8,
        #[max_length(16)]
        level_type: String,
        #[var_int]
        view_distance: i32,
        reduced_debug: bool,
        enable_respawn_screen: bool,
    }

    impl From<JoinGameSpec> for JoinGamePacket {
        fn from(spec: JoinGameSpec) -> Self {
            JoinGamePacket {
                entity_id: spec.entity_id,
                game_mode: spec.game_mode as u8,
                dimension: spec.dimension,
                hashed_seed: spec.hashed_seed,
                max_players: spec.max_players,
                level_type: spec.level_type,
                view_distance: spec.view_distance,
                reduced_debug: spec.reduced_debug,
                enable_respawn_screen: spec.enable_respawn_screen,
            }
        }
    }
}