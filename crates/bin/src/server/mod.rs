use std::collections::hash_map::Entry;
use std::io::Read;
use std::thread;
use std::time::Duration;

use ahash::AHashMap;
use fastnbt::de::from_bytes;
use flate2::read::GzDecoder;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::MissedTickBehavior;
use uuid::Uuid;

use falcon_core::errors::ResultExt;
use falcon_core::network::connection::ConnectionTask;
use falcon_core::player::MinecraftPlayer;
use falcon_core::schematic::{SchematicData, SchematicVersionedRaw};
use falcon_core::server::config::FalconConfig;
use falcon_core::server::{Difficulty, McTask, MinecraftServer};
use falcon_core::world::chunks::Chunk;
use falcon_core::world::World;
use falcon_core::ShutdownHandle;
use falcon_protocol::ProtocolSend;

use crate::errors::*;
use crate::network::NetworkListener;
use crate::player::Player;
use crate::server::console::ConsoleListener;

pub mod console;

pub struct MainServer {
    // threads
    shutdown_handle: ShutdownHandle,
    should_stop: bool,
    console_rx: UnboundedReceiver<String>,
    server_rx: UnboundedReceiver<Box<McTask>>,
    // players
    entity_id_count: i32,
    players: AHashMap<Uuid, Player>,
    // world
    world: World,
}

/// Initialization methods on startup
impl MainServer {
    pub fn start_server(shutdown_handle: ShutdownHandle) -> Result<()> {
        info!("Starting server thread...");

        let world = {
            let world_file = std::fs::read("./world.schem")
                .chain_err(|| "Could not load `world.schem`, stopping launch")?;
            let mut gz = GzDecoder::new(&world_file[..]);
            let mut decompressed_world = Vec::new();
            gz.read_to_end(&mut decompressed_world)
                .chain_err(|| "Could not decompress world.schem, is this a valid schematic?")?;
            debug!("Checkpoint - loaded schem file");
            let schematic: SchematicVersionedRaw = from_bytes(&decompressed_world)
                .chain_err(|| "Could not parse schematic file, is this valid nbt?")?;
            debug!("Checkpoint - deserialized into raw format");
            let data = SchematicData::try_from(schematic)
                .chain_err(|| "Invalid schematic, this server cannot use this schematic currently!")?;
            debug!("Checkpoint - parsed raw format");
            World::try_from(data)?
        };
        info!("Loaded world");

        let console_rx = ConsoleListener::start_console(shutdown_handle.clone())?;
        let (server_tx, server_rx) = unbounded_channel();
        let server = MainServer {
            shutdown_handle,
            should_stop: false,
            console_rx,
            server_rx,
            entity_id_count: 0,
            players: AHashMap::new(),
            world,
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
}

/// Game loop methods
impl MainServer {
    #[tracing::instrument(name = "server", skip(self))]
    fn start_server_logic(mut self) {
        debug!("Starting server logic!");
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_millis(50));
            tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let mut keep_alive_interval = tokio::time::interval(Duration::from_secs(12));
            keep_alive_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            while !self.should_stop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        self.tick();
                    }
                    _ = keep_alive_interval.tick() => {
                        self.keep_alive();
                    }
                    _ = self.shutdown_handle.wait_for_shutdown() => {
                        break;
                    }
                }
            }
            debug!("Stopping server logic!");
        });
    }

    #[tracing::instrument(skip(self), fields(player_count = self.players.len()))]
    fn tick(&mut self) {
        while let Ok(task) = self.server_rx.try_recv() {
            task(self);
        }
        while let Ok(command) = self.console_rx.try_recv() {
            info!(cmd = %command.trim(), "Console command execution");
            // TODO: better commands
            if command.trim() == "stop" {
                info!("Shutting down server! (Stop command executed)");
                self.should_stop = true;
                self.shutdown_handle.send_shutdown();
                return;
            }
        }
    }

    #[tracing::instrument(skip(self), fields(player_count = self.players.len()))]
    fn keep_alive(&mut self) {
        self.players.retain(|_, player| player.send_keep_alive().is_ok());
    }
}

impl MinecraftServer for MainServer {
    fn online_player_count(&self) -> i32 {
        self.players.len() as i32
    }

    fn get_player(&self, uuid: Uuid) -> Option<&dyn MinecraftPlayer> {
        self.players.get(&uuid).map(|player| player as &dyn MinecraftPlayer)
    }

    fn get_player_mut(&mut self, uuid: Uuid) -> Option<&mut dyn MinecraftPlayer> {
        self.players.get_mut(&uuid).map(|player| player as &mut dyn MinecraftPlayer)
    }

    fn player_join(&mut self, username: String, uuid: uuid::Uuid, protocol_version: i32, client_connection: UnboundedSender<Box<ConnectionTask>>) {
        if self.players.contains_key(&uuid) {
            // TODO: Kick duplicqted playeers
            error!(%uuid, %username, "Duplicate player joining");
        }
        info!(name = %username, "Player joined the game!");
        let (spawn_pos, spawn_look) = (FalconConfig::global().spawn_pos(), FalconConfig::global().spawn_look());
        let player = Player::new(username, uuid, self.entity_id_count, spawn_pos, spawn_look, protocol_version, client_connection);
        self.entity_id_count += 1;

        let player = self.players.entry(uuid).or_insert(player);
        if let Err(error) = ProtocolSend::join_game(player, Difficulty::Peaceful, FalconConfig::global().max_players() as u8, String::from("customized"), 6, false) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }
        if let Err(error) = ProtocolSend::player_abilities(player, 0.05, 0.1) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }

        let chunk_fn = |player: &mut dyn MinecraftPlayer, chunk: &Chunk| {
            ProtocolSend::send_chunk(player, chunk).chain_err(|| "Error sending chunk")
        };
        let chunk_air_fn = |player: &mut dyn MinecraftPlayer, x: i32, z: i32| {
            ProtocolSend::send_air_chunk(player, x, z).chain_err(|| "Error sending chunk")
        };
        if let Err(error) = self.world.send_chunks_for_player(player, chunk_fn, chunk_air_fn) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }

        if let Err(error) = ProtocolSend::player_position_and_look(player, 0, 1) {
            player.disconnect(format!("Error whilst sending packet: {}", error));
        }
    }

    fn player_leave(&mut self, uuid: Uuid) {
        let player = self.players.remove(&uuid);
        if let Some(player) = player {
            info!(%uuid, name = player.get_username(), "Player disconnected!");
        }
    }

    /// Signals through the update of the player's position and look
    fn player_position_and_look(
        &mut self,
        uuid: Uuid,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        _on_ground: bool,
    ) {
        // TODO: fire event
        if let Entry::Occupied(mut entry) = self.players.entry(uuid) {
            let player = entry.get_mut();
            let position = player.get_position_mut();
            let (old_chunk_x, old_chunk_z) = (position.get_chunk_x(), position.get_chunk_z());
            position.set_x(x);
            position.set_y(y);
            position.set_z(z);
            let (chunk_x, chunk_z) = (position.get_chunk_x(), position.get_chunk_z());
            let look_angles = player.get_look_angles_mut();
            look_angles.set_yaw(yaw);
            look_angles.set_pitch(pitch);

            if chunk_x != old_chunk_x || chunk_z != old_chunk_z {
                let chunk_fn = |player: &mut dyn MinecraftPlayer, chunk: &Chunk| {
                    ProtocolSend::send_chunk(player, chunk).chain_err(|| "Error sending chunk")
                };
                let chunk_air_fn = |player: &mut dyn MinecraftPlayer, x: i32, z: i32| {
                    ProtocolSend::send_air_chunk(player, x, z).chain_err(|| "Error sending chunk")
                };
                let unload_fn = |player: &mut dyn MinecraftPlayer, x: i32, z: i32| {
                    ProtocolSend::unload_chunk(player, x, z).chain_err(|| "Erorr unloading chunk")
                };
                if let Err(error) = self.world.update_player_pos(player, old_chunk_x, old_chunk_z, chunk_x, chunk_z, chunk_fn, chunk_air_fn, unload_fn) {
                    player.disconnect(format!("Error whilst sending packet: {}", error));
                }
            }
        }
    }

    fn player_update_view_distance(&mut self, uuid: Uuid, view_distance: u8) {
        if let Entry::Occupied(mut entry) = self.players.entry(uuid) {
            let player = entry.get_mut();
            let chunk_fn = |player: &mut dyn MinecraftPlayer, chunk: &Chunk| {
                ProtocolSend::send_chunk(player, chunk).chain_err(|| "Error sending chunk")
            };
            let chunk_air_fn = |player: &mut dyn MinecraftPlayer, x: i32, z: i32| {
                ProtocolSend::send_air_chunk(player, x, z).chain_err(|| "Error sending chunk")
            };
            let unload_fn = |player: &mut dyn MinecraftPlayer, x: i32, z: i32| {
                ProtocolSend::unload_chunk(player, x, z).chain_err(|| "Erorr unloading chunk")
            };
            if let Err(error) = self.world.update_view_distance(player, view_distance, chunk_fn, chunk_air_fn, unload_fn) {
                player.disconnect(format!("Error whilst sending packet: {}", error));
            }
            player.set_view_distance(view_distance);
        }
    }
}
