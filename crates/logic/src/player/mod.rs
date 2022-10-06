use falcon_core::player::data::{GameMode, LookAngles, PlayerAbilityFlags, Position};
use falcon_core::server::config::FalconConfig;
use falcon_core::server::data::Difficulty;
use falcon_send::specs::play::JoinGameSpec;
use mc_chat::ChatComponent;
use tokio::time::Instant;
use uuid::Uuid;

use crate::connection::ConnectionWrapper;

#[derive(Debug)]
pub struct FalconPlayer {
    // identity
    username: String,
    uuid: Uuid,
    // in-game
    eid: i32,
    gamemode: GameMode,
    dimension: i32,
    abilities: PlayerAbilityFlags,
    position: Position,
    facing: LookAngles,
    view_distance: u8,
    // network
    time: Instant,
    protocol: i32,
    connection: ConnectionWrapper,
}

impl FalconPlayer {
    pub fn new(username: String, uuid: Uuid, eid: i32, pos: Position, facing: LookAngles, protocol: i32, connection: ConnectionWrapper) -> Self {
        FalconPlayer {
            username,
            uuid,
            eid,
            gamemode: GameMode::Creative,
            dimension: 0,
            abilities: PlayerAbilityFlags::new(false, true, true, true),
            position: pos,
            facing,
            view_distance: 5,
            time: Instant::now(),
            protocol,
            connection,
        }
    }

    pub fn username(&self) -> &str { &self.username }

    pub fn uuid(&self) -> Uuid { self.uuid }

    pub fn entity_id(&self) -> i32 { self.eid }

    pub fn gamemode(&self) -> GameMode { self.gamemode }

    pub fn dimension(&self) -> i32 { self.dimension }

    pub fn ability_flags(&self) -> PlayerAbilityFlags { self.abilities }

    pub fn position(&self) -> &Position { &self.position }

    pub fn position_mut(&mut self) -> &mut Position { &mut self.position }

    pub fn look_angles(&self) -> &LookAngles { &self.facing }

    pub fn look_angles_mut(&mut self) -> &mut LookAngles { &mut self.facing }

    pub fn view_distance(&self) -> u8 { self.view_distance }

    pub fn set_view_distance(&mut self, distance: u8) {
        self.view_distance = std::cmp::max(0, std::cmp::min(distance, FalconConfig::global().max_view_distance()));
    }

    pub fn protocol_version(&self) -> i32 { self.protocol }

    pub fn connection(&self) -> &ConnectionWrapper { &self.connection }
}

impl FalconPlayer {
    pub fn disconnect(&mut self, reason: ChatComponent) { self.connection.execute_sync(move |connection| Ok(connection.disconnect(reason))); }

    #[tracing::instrument(skip(self))]
    pub fn send_keep_alive(&self) {
        let elapsed = self.time.elapsed().as_secs();
        self.connection.execute_sync(move |connection| {
            connection.handler_state_mut().set_last_keep_alive(elapsed);

            Ok(connection.send_packet(elapsed as i64, falcon_send::write_keep_alive)?)
        });
    }

    pub fn join_spec(&self, difficulty: Difficulty, max_players: u8, level_type: String, seed: i64, reduced_debug: bool, enable_respawn: bool) -> JoinGameSpec {
        JoinGameSpec::new(
            self.eid,
            self.gamemode,
            self.dimension,
            difficulty,
            max_players,
            level_type,
            seed,
            self.view_distance as i32,
            reduced_debug,
            enable_respawn,
        )
    }
}
