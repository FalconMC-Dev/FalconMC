use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use falcon_core::network::connection::{ConnectionTask, MinecraftConnection};
use falcon_core::player::{LookAngles, MinecraftPlayer, PlayerAbilityFlags, Position};
use falcon_core::player::GameMode;

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
    protocol_version: i32,
    connection: UnboundedSender<Box<ConnectionTask>>,
}

impl Player {
    pub fn new(username: String, uuid: Uuid, eid: i32, protocol_version: i32, connection: UnboundedSender<Box<ConnectionTask>>) -> Self {
        Player {
            username,
            uuid,
            eid,
            game_mode: GameMode::Creative,
            dimension: 0,
            ability_flags: PlayerAbilityFlags::new(false, true, true, true),
            position: Default::default(),
            look_angles: Default::default(),
            protocol_version,
            connection,
        }
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
                conn.disconnect(reason);
            })
        };
        if let Err(error) = self.connection.send(task) {
            error!("Could not disconnect player, some witchcraft must be happening here ({})!", error);
        }
    }

    fn get_client_connection(&mut self) -> &mut UnboundedSender<Box<ConnectionTask>> {
        &mut self.connection
    }
}