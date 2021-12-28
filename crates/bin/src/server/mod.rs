use std::io::Read;
use std::thread;
use std::time::Duration;

use ahash::AHashMap;
use crossbeam::channel::Receiver;
use fastnbt::de::from_bytes;
use flate2::read::GzDecoder;
use tokio::runtime::Builder;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::MissedTickBehavior;
use uuid::Uuid;

use falcon_core::errors::ResultExt;
use falcon_core::network::connection::ConnectionTask;
use falcon_core::player::MinecraftPlayer;
use falcon_core::schematic::{SchematicData, SchematicVersionedRaw};
use falcon_core::server::{Difficulty, McTask, MinecraftServer};
use falcon_core::server::config::FalconConfig;
use falcon_core::ShutdownHandle;
use falcon_core::world::chunks::Chunk;
use falcon_core::world::World;
use falcon_protocol::ProtocolSend;

use crate::errors::*;
use crate::network::listener::NetworkListener;
use crate::player::Player;

pub struct MainServer {
    // threads
    shutdown_handle: ShutdownHandle,
    server_rx: Receiver<Box<McTask>>,
    // players
    entity_id_count: i32,
    players: AHashMap<Uuid, Player>,
    // world
    world: World,
}

impl MainServer {
    pub fn start_server(shutdown_handle: ShutdownHandle) -> Result<()> {
        info!("Starting server thread...");

        let world_file = std::fs::read("./world.schem").chain_err(|| "Could not load `world.schem`, stopping launch")?;
        let mut gz = GzDecoder::new(&world_file[..]);
        let mut decompressed_world = Vec::new();
        gz.read_to_end(&mut decompressed_world).chain_err(|| "Could not decompress world.schem, is this a valid schematic?")?;
        let schematic: SchematicVersionedRaw = from_bytes(&decompressed_world).chain_err(|| "Could not parse schematic file, is this valid nbt?")?;
        let world = World::try_from(SchematicData::try_from(schematic).chain_err(|| "Invalid schematic, this server cannot use this schematic currently!")?)?;
        info!("Loaded world");

        let (server_tx, server_rx) = crossbeam::channel::unbounded();
        let server = MainServer {
            shutdown_handle,
            server_rx,
            entity_id_count: 0,
            players: AHashMap::new(),
            world
        };

        tokio::spawn(NetworkListener::start_network_listening(
            server.shutdown_handle.clone(),
            server_tx,
        ));

        thread::Builder::new()
            .name(String::from("Main Server Thread"))
            .spawn(|| server.start_server_logic())
            .chain_err(|| "Couldn't start server logic!")?;

        Ok(())
    }

    fn start_server_logic(mut self) {
        debug!("Starting server logic!");
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_millis(50));
            tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let mut keep_alive_interval = tokio::time::interval(Duration::from_secs(12));
            keep_alive_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        self.tick();
                    }
                    _ = keep_alive_interval.tick() => {
                        self.keep_alive();
                    }
                    _ = self.shutdown_handle.wait_for_shutdown() => {
                        debug!("Stopping server logic!");
                        break;
                    }
                }
            }
        });
    }

    fn tick(&mut self) {
        while let Ok(task) = self.server_rx.try_recv() {
            task(self);
        }
    }

    fn keep_alive(&mut self) {
        self.players.retain(|_, player| player.send_keep_alive().is_ok());
    }
}

impl MinecraftServer for MainServer {
    fn get_player(&self, uuid: Uuid) -> Option<&dyn MinecraftPlayer> {
        self.players.get(&uuid).map(|player| player as &dyn MinecraftPlayer)
    }

    fn get_player_mut(&mut self, uuid: Uuid) -> Option<&mut dyn MinecraftPlayer> {
        self.players.get_mut(&uuid).map(|player| player as &mut dyn MinecraftPlayer)
    }

    fn player_join(&mut self, username: String, uuid: uuid::Uuid, protocol_version: i32, client_connection: UnboundedSender<Box<ConnectionTask>>) {
        if self.players.contains_key(&uuid) {
            error!("Player already exists! {}: {}", &username, uuid);
        }
        info!("{} Joined the game!", username);
        let player = Player::new(username, uuid, self.entity_id_count, protocol_version, client_connection);
        self.entity_id_count += 1;
        trace!("New player: {:?}", player);

        self.players.insert(uuid, player);
        let player = self.players.get_mut(&uuid).expect("Should always get the player we just put in");
        if let Err(error) = ProtocolSend::join_game(player, Difficulty::Peaceful, FalconConfig::global().max_players() as u8, String::from("customized"), false) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }
        if let Err(error) = ProtocolSend::player_abilities(player, 0.05, 0.1) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }

        let chunk_fn = {
            |player: &mut dyn MinecraftPlayer, chunk: &Chunk| ProtocolSend::send_chunk(player, chunk).chain_err(|| "Error sending chunk")
        };
        let chunk_air_fn = {
            |player: &mut dyn MinecraftPlayer, x: i32, z: i32| ProtocolSend::send_air_chunk(player, x, z).chain_err(|| "Error sending chunk")
        };
        if let Err(error) = self.world.send_chunks_for_player(player, chunk_fn, chunk_air_fn) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }

        if let Err(error) = ProtocolSend::player_position_and_look(player, 0, 1) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }
    }

    fn player_leave(&mut self, uuid: Uuid) {
        self.players.remove(&uuid);
    }

    /// Signals through the update of the player's position and look
    fn player_position_and_look(&mut self, uuid: Uuid, x: f64, y: f64, z: f64, yaw: f32, pitch: f32, _on_ground: bool) {
        // TODO: fire event
        if let Some(player) = self.get_player_mut(uuid) {
            let position = player.get_position_mut();
            position.set_x(x);
            position.set_y(y);
            position.set_z(z);
            let look_angles = player.get_look_angles_mut();
            look_angles.set_yaw(yaw);
            look_angles.set_pitch(pitch);
        }
    }
}
