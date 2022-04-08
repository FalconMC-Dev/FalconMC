use std::time::Instant;

use uuid::Uuid;

use data::*;

use crate::network::connection::ConnectionWrapper;
use crate::server::config::FalconConfig;

pub mod data;

#[derive(Debug)]
pub struct Player {
    // identity
    username: String,
    uuid: Uuid,
    // in-game data
    eid: i32,
    game_mode: GameMode,
    dimension: i32,
    ability_flags: PlayerAbilityFlags,
    position: Position,
    look_angles: LookAngles,
    view_distance: u8,
    // connection data
    pub time: Instant,
    protocol_version: i32,
    connection: ConnectionWrapper,
}

impl Player {
    pub fn new(
        username: String,
        uuid: Uuid,
        eid: i32,
        spawn_pos: Position,
        spawn_look: LookAngles,
        protocol_version: i32,
        connection: ConnectionWrapper,
    ) -> Self {
        Player {
            username,
            uuid,
            eid,
            game_mode: GameMode::Creative,
            dimension: 0,
            ability_flags: PlayerAbilityFlags::new(false, true, true, true),
            position: spawn_pos,
            look_angles: spawn_look,
            view_distance: 5,
            time: Instant::now(),
            protocol_version,
            connection,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn entity_id(&self) -> i32 {
        self.eid
    }

    pub fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    pub fn dimension(&self) -> i32 {
        self.dimension
    }

    pub fn ability_flags(&self) -> PlayerAbilityFlags {
        self.ability_flags
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    pub fn look_angles(&self) -> &LookAngles {
        &self.look_angles
    }

    pub fn look_angles_mut(&mut self) -> &mut LookAngles {
        &mut self.look_angles
    }

    pub fn view_distance(&self) -> u8 {
        self.view_distance
    }

    pub fn set_view_distance(&mut self, distance: u8) {
        self.view_distance = std::cmp::max(0, std::cmp::min(distance, FalconConfig::global().max_view_distance()));
    }

    pub fn protocol_version(&self) -> i32 {
        self.protocol_version
    }

    pub fn connection(&self) -> &ConnectionWrapper {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut ConnectionWrapper {
        &mut self.connection
    }
}
