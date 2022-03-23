use std::time::Instant;

use uuid::Uuid;

use falcon_core::network::connection::{ConnectionActor, ConnectionWrapper};
use falcon_core::player::GameMode;
use falcon_core::player::{LookAngles, MinecraftPlayer, PlayerAbilityFlags, Position};
use falcon_core::server::config::FalconConfig;
use falcon_protocol::ProtocolSend;

use anyhow::Result;

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
    time: Instant,
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

    #[tracing::instrument(skip(self))]
    pub fn send_keep_alive(&mut self) -> Result<()> {
        debug!("Keep alive sent!");
        let elapsed = self.time.elapsed();
        if let Err(error) = ProtocolSend::keep_alive(self, elapsed.as_secs()) {
            bail!("Could not send keep alive - disconnecting! For: {}, due to: {}", &self.username, error);
        }
        Ok(())
    }
}

impl MinecraftPlayer for Player {
    fn username(&self) -> &str {
        &self.username
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn entity_id(&self) -> i32 {
        self.eid
    }

    fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    fn dimension(&self) -> i32 {
        self.dimension
    }

    fn ability_flags(&self) -> PlayerAbilityFlags {
        self.ability_flags
    }

    fn position(&self) -> &Position {
        &self.position
    }

    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    fn look_angles(&self) -> &LookAngles {
        &self.look_angles
    }

    fn look_angles_mut(&mut self) -> &mut LookAngles {
        &mut self.look_angles
    }

    fn view_distance(&self) -> u8 {
        self.view_distance
    }

    fn set_view_distance(&mut self, distance: u8) {
        self.view_distance = std::cmp::max(0, std::cmp::min(distance, FalconConfig::global().max_view_distance()));
        debug!(view_distance = self.view_distance, "Decided view-distance");
    }

    fn protocol_version(&self) -> i32 {
        self.protocol_version
    }

    fn disconnect(&mut self, reason: String) {
        self.connection.disconnect(reason);
    }

    fn client_connection(&mut self) -> &mut ConnectionWrapper {
        &mut self.connection
    }
}
