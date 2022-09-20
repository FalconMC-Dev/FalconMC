mod chunk;
pub mod dimension;

pub use chunk::*;

#[falcon_send_derive::falcon_send]
mod inner {
    use crate::v1_16::play::dimension::{Codec, DimensionData};
    use crate::JoinGameSpec;
    use bytes::BufMut;
    use derive_from_ext::From;
    use falcon_core::data::Identifier;
    use falcon_core::world::dimension::Dimension;
    use falcon_packet_core::{
        PacketSize, PacketSizeSeed, PacketString, PacketWrite, PacketWriteSeed, WriteError,
    };

    #[derive(PacketSize, PacketWrite, From)]
    #[from(JoinGameSpec)]
    #[falcon_packet(versions = {
        735, 736 = 0x25;
    }, name = "join_game")]
    pub struct JoinGamePacket {
        entity_id: i32,
        game_mode: u8,
        #[from(rename = "game_mode")]
        prev_gamemode: u8,
        #[from(skip)]
        #[falcon(var32)]
        world_count: usize,
        #[from(skip, default = "init_worlds()")]
        #[falcon(link = "world_count with worlds")]
        worlds: Vec<Identifier>,
        #[from(skip, default = "init_dimension_codec()")]
        #[falcon(nbt)]
        dimension_codec: Codec,
        #[from(skip, default = "init_dimension()")]
        #[falcon(to_string)]
        dimention: Identifier,
        #[from(skip, default = "init_world()")]
        #[falcon(to_string)]
        world_name: Identifier,
        hashed_seed: i64,
        max_players: u8,
        #[falcon(var32)]
        view_distance: i32,
        reduced_debug: bool,
        enable_respawn_screen: bool,
        #[from(skip)]
        is_debug: bool,
        #[from(skip)]
        is_flat: bool,
    }

    fn worlds_value(field: &Vec<Identifier>) -> usize {
        field.len()
    }

    fn worlds_size(field: &Vec<Identifier>) -> usize {
        field
            .iter()
            .map(|i| PacketSizeSeed::size(&PacketString::new(32767), &i.to_string()))
            .sum::<usize>()
    }

    fn worlds_write<B>(field: Vec<Identifier>, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        for ident in field {
            PacketWriteSeed::write(PacketString::new(32767), ident.to_string(), buffer)?;
        }
        Ok(())
    }

    fn init_worlds() -> Vec<Identifier> {
        vec![init_world()]
    }

    fn init_world() -> Identifier {
        Identifier::from_static("falcon", "world")
    }

    fn init_dimension_codec() -> Codec {
        Codec::new(vec![DimensionData::new(Dimension::new(
            "minecraft:overworld",
            0,
        ))])
    }

    fn init_dimension() -> Identifier {
        Identifier::from_static("minecraft", "overworld")
    }
}
