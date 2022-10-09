#[falcon_send_derive::falcon_send]
mod inner {
    use derive_from_ext::From;
    use falcon_packet_core::{PacketSize, PacketWrite};
    use mc_chat::ChatComponent;

    use crate::specs::play::{JoinGameSpec, PlayerAbilitiesSpec};
    use crate::ServerDifficultySpec;

    #[derive(PacketSize, PacketWrite, From)]
    #[from(JoinGameSpec)]
    #[falcon_packet(versions = {
        47 = 0x01;
        107 = 0x23;
    }, name = "join_game")]
    pub struct JoinGamePacket {
        entity_id: i32,
        game_mode: u8,
        #[from(map = "i32_to_i8")]
        dimension: i8,
        difficulty: u8,
        max_players: u8,
        #[falcon(string)]
        level_type: String,
        reduced_debug: bool,
    }

    fn i32_to_i8(n: i32) -> i8 { n as i8 }

    #[derive(PacketSize, PacketWrite, From)]
    #[from(PlayerAbilitiesSpec)]
    #[falcon_packet(versions = {
        47 = 0x39;
        107, 108, 109, 110, 210, 315, 316, 335 = 0x2B;
        338, 340 = 0x2C;
        393, 401, 404 = 0x2E;
        477, 480, 485, 490, 498, 735, 736 = 0x31;
        573, 575, 578 = 0x32;
    }, name = "player_abilities")]
    pub struct PlayerAbilityPacket {
        flags: u8,
        flying_speed: f32,
        fov_modifier: f32,
    }

    #[derive(PacketSize, PacketWrite)]
    #[falcon_packet(versions = {
        47 = 0x40;
        107, 108, 109, 110, 210, 315, 316, 335, 338, 340, 477, 480, 485, 490, 498, 735, 736 = 0x1A;
        393, 401, 404, 573, 575, 578 = 0x1B;
    }, name = "disconnect")]
    pub struct DisconnectPacket {
        #[falcon(string = 262144)]
        reason: String,
    }

    impl From<ChatComponent> for DisconnectPacket {
        fn from(reason: ChatComponent) -> Self {
            DisconnectPacket {
                reason: serde_json::to_string(&reason).expect("Invalid reason data"),
            }
        }
    }

    #[derive(PacketSize, PacketWrite, From)]
    #[from(ServerDifficultySpec)]
    #[falcon_packet(versions = {
        47 = 0x41;
        107, 108, 109, 110, 210, 315, 316, 335, 338, 340, 393, 401, 404 = 0x0D;
    }, name = "difficulty")]
    pub struct ServerDifficultyPacket {
        difficulty: u8,
    }
}
