#[falcon_send_derive::falcon_send]
mod inner {
    use derive_from_ext::From;
    use falcon_packet_core::{PacketSize, PacketWrite};

    use crate::specs::play::JoinGameSpec;

    #[derive(PacketSize, PacketWrite, From)]
    #[from(JoinGameSpec)]
    #[falcon_packet(versions = {
        108, 109, 110, 210, 315, 316, 335, 338, 340 = 0x23;
        393, 401, 404 = 0x25;
    }, name = "join_game")]
    pub struct JoinGamePacket {
        entity_id: i32,
        game_mode: u8,
        dimension: i32,
        difficulty: u8,
        max_players: u8,
        #[falcon(string = 16)]
        level_type: String,
        reduced_debug: bool,
    }
}
