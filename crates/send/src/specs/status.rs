use crate::define_spec;
use falcon_core::server::data::ServerVersion;
use serde::Serialize;

define_spec! {
    StatusResponseSpec {
        version: ServerVersion,
        players: PlayerData,
        description: String,
    }, Debug, Serialize
}

define_spec! {
    PlayerData {
        max: i32,
        online: i32,
    }, Debug, Serialize
}
