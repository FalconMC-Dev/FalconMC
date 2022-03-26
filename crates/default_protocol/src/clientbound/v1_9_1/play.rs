pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::packet::PacketEncode;
    use crate::clientbound::specs::play::JoinGameSpec;

    #[derive(PacketEncode)]
    #[falcon_packet(108, 109, 110, 210, 315, 316, 335, 338, 340 = 0x23; 393, 401, 404 = 0x25; no_receive; outgoing = "join_game")]
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

    impl From<JoinGameSpec> for JoinGamePacket {
        fn from(spec: JoinGameSpec) -> Self {
            JoinGamePacket {
                entity_id: spec.entity_id,
                game_mode: spec.game_mode as u8,
                dimension: spec.dimension as i32,
                difficulty: spec.difficulty as u8,
                max_players: spec.max_players,
                level_type: spec.level_type,
                reduced_debug: spec.reduced_debug
            }
        }
    }
}