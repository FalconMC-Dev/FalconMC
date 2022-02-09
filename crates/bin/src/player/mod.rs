use std::time::Instant;

use ignore_result::Ignore;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use falcon_core::network::connection::{ConnectionTask, MinecraftConnection};
use falcon_core::player::{LookAngles, MinecraftPlayer, PlayerAbilityFlags, Position};
use falcon_core::player::GameMode;
use falcon_protocol::ProtocolSend;

use crate::errors::*;

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
    // connection data
    time: Instant,
    protocol_version: i32,
    connection: UnboundedSender<Box<ConnectionTask>>,
}

impl Player {
    pub fn new(username: String, uuid: Uuid, eid: i32, spawn_pos: Position, spawn_look: LookAngles, protocol_version: i32, connection: UnboundedSender<Box<ConnectionTask>>) -> Self {
        Player {
            username,
            uuid,
            eid,
            game_mode: GameMode::Creative,
            dimension: 0,
            ability_flags: PlayerAbilityFlags::new(false, true, true, true),
            position: spawn_pos,
            look_angles: spawn_look,
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
    fn get_username(&self) -> &str {
        &self.username
    }

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    fn get_entity_id(&self) -> i32 {
        self.eid
    }

    fn get_game_mode(&self) -> GameMode {
        self.game_mode
    }

    fn get_dimension(&self) -> i32 {
        self.dimension
    }

    fn get_ability_flags(&self) -> PlayerAbilityFlags {
        self.ability_flags
    }

    fn get_position(&self) -> &Position {
        &self.position
    }

    fn get_position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    fn get_look_angles(&self) -> &LookAngles {
        &self.look_angles
    }

    fn get_look_angles_mut(&mut self) -> &mut LookAngles {
        &mut self.look_angles
    }

    fn get_protocol_version(&self) -> i32 {
        self.protocol_version
    }

    fn disconnect(&mut self, reason: String) {
        let task = {
            // TODO: Using login packet here, this is incorrect
            Box::new(move |conn: &mut dyn MinecraftConnection| {
                conn.disconnect(reason)
            })
        };
        self.connection.send(task).ignore();
    }

    fn get_client_connection(&mut self) -> &mut UnboundedSender<Box<ConnectionTask>> {
        &mut self.connection
    }
}