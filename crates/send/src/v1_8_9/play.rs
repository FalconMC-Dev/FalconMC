pub use inner::*;

#[falcon_protocol_derive::packet_module]
mod inner {
    use mc_chat::ChatComponent;
    use falcon_core::network::packet::PacketEncode;
    use falcon_core::player::data::PlayerAbilityFlags;
    use crate::ServerDifficultySpec;
    use crate::specs::play::{JoinGameSpec, PlayerAbilitiesSpec};

    #[derive(PacketEncode)]
    #[falcon_packet(47 = 0x01; 107 = 0x23; no_receive; outgoing = "join_game")]
    pub struct JoinGamePacket {
        entity_id: i32,
        game_mode: u8,
        dimension: i8,
        difficulty: u8,
        max_players: u8,
        level_type: String,
        reduced_debug: bool,
    }

    impl From<JoinGameSpec> for JoinGamePacket {
        fn from(spec: JoinGameSpec) -> Self {
            JoinGamePacket {
                entity_id: spec.entity_id,
                game_mode: spec.game_mode as u8,
                dimension: spec.dimension as i8,
                difficulty: spec.difficulty as u8,
                max_players: spec.max_players,
                level_type: spec.level_type,
                reduced_debug: spec.reduced_debug
            }
        }
    }

    #[derive(PacketEncode)]
    #[falcon_packet(47 = 0x39; 107, 108, 109, 110, 210, 315, 316, 335 = 0x2B; 338, 340 = 0x2C; 393, 401, 404 = 0x2E; 477, 480, 485, 490, 498 = 0x31; no_receive; outgoing = "player_abilities")]
    pub struct PlayerAbilityPacket {
        flags: PlayerAbilityFlags,
        fly_speed: f32,
        fov_modifier: f32,
    }

    impl From<PlayerAbilitiesSpec> for PlayerAbilityPacket {
        fn from(spec: PlayerAbilitiesSpec) -> Self {
            PlayerAbilityPacket {
                flags: spec.flags,
                fly_speed: spec.flying_speed,
                fov_modifier: spec.fov_modifier
            }
        }
    }

    #[derive(PacketEncode)]
    #[falcon_packet(47 = 0x40; 107, 108, 109, 110, 210, 315, 316, 335, 338, 340, 477, 480, 485, 490, 498 = 0x1A; 393, 401, 404 = 0x1B; no_receive; outgoing = "disconnect")]
    pub struct DisconnectPacket {
        #[max_length(262144)]
        reason: String,
    }

    impl From<ChatComponent> for DisconnectPacket {
        fn from(reason: ChatComponent) -> Self {
            DisconnectPacket {
                reason: serde_json::to_string(&reason).unwrap(),
            }
        }
    }

    #[derive(PacketEncode)]
    #[falcon_packet(47 = 0x41; 107, 108, 109, 110, 210, 315, 316, 335, 338, 340, 393, 401, 404 = 0x0D; no_receive; outgoing = "send_difficulty")]
    pub struct ServerDifficultyPacket {
        difficulty: u8,
    }

    impl From<ServerDifficultySpec> for ServerDifficultyPacket {
        fn from(spec: ServerDifficultySpec) -> Self {
            ServerDifficultyPacket {
                difficulty: spec.difficulty as u8,
            }
        }
    }
}