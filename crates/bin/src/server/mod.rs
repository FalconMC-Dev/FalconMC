use std::collections::hash_map::Entry;
use std::io::Read;
use std::thread;
use std::time::Duration;

use ahash::AHashMap;
use anyhow::{Result, Context};
use fastnbt::de::from_bytes;
use flate2::read::GzDecoder;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::time::MissedTickBehavior;
use uuid::Uuid;

use falcon_core::network::connection::ConnectionWrapper;
use falcon_core::network::ConnectionState;
use falcon_core::player::MinecraftPlayer;
use falcon_core::schematic::{SchematicData, SchematicVersionedRaw};
use falcon_core::server::config::FalconConfig;
use falcon_core::server::{Difficulty, McTask, ServerActor, ServerData, ServerVersion};
use falcon_core::world::chunks::Chunk;
use falcon_core::world::World;
use falcon_core::ShutdownHandle;
use falcon_default_protocol::clientbound as falcon_send;
use falcon_default_protocol::clientbound::specs::play::{ChunkDataSpec, JoinGameSpec, PlayerAbilitiesSpec, PositionAndLookSpec};
use falcon_default_protocol::clientbound::specs::status::{PlayerData, StatusResponseSpec};
use falcon_send::specs::login::LoginSuccessSpec;

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
                .with_context(|| "Could not load `world.schem`, stopping launch")?;
            let mut gz = GzDecoder::new(&world_file[..]);
            let mut decompressed_world = Vec::new();
            gz.read_to_end(&mut decompressed_world)
                .with_context(|| "Could not decompress world.schem, is this a valid schematic?")?;
            debug!("Checkpoint - loaded schem file");
            let schematic: SchematicVersionedRaw = from_bytes(&decompressed_world)
                .with_context(|| "Could not parse schematic file, is this valid nbt?")?;
            debug!("Checkpoint - deserialized into raw format");
            let data = SchematicData::try_from(schematic)
                .with_context(|| "Invalid schematic, this server cannot use this schematic currently!")?;
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
            .with_context(|| "Couldn't start server logic!")?;

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
        self.players.values().for_each(|player| player.send_keep_alive());
    }
}

impl ServerData for MainServer {
    fn online_count(&self) -> i32 {
        self.players.len() as i32
    }

    fn player(&self, uuid: Uuid) -> Option<&dyn MinecraftPlayer> {
        self.players.get(&uuid).map(|player| player as &dyn MinecraftPlayer)
    }

    fn player_mut(&mut self, uuid: Uuid) -> Option<&mut dyn MinecraftPlayer> {
        self.players.get_mut(&uuid).map(|player| player as &mut dyn MinecraftPlayer)
    }
}

impl ServerActor for MainServer {
    fn request_status(&self, protocol_id: i32, connection: ConnectionWrapper) {
        let version = ServerVersion::new(String::from("1.13-1.17.1"), protocol_id);
        let player_data = PlayerData::new(FalconConfig::global().max_players(), self.online_count());
        let description = String::from("§eFalcon server§r§b!!!");
        connection.build_send_packet(StatusResponseSpec::new(version, player_data, description), |p, c| falcon_send::send_status_response(p, c));
    }

    fn player_login(&mut self, username: String, protocol_version: i32, client_connection: ConnectionWrapper) {
        debug!(player_name = %username);
        let player_uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, username.as_bytes());
        let username2 = username.clone();
        client_connection.execute(move |connection| {
            falcon_send::send_login_success(LoginSuccessSpec::new(player_uuid, username2), connection);
            let handler_state = connection.handler_state_mut();
            handler_state.set_connection_state(ConnectionState::Play);
            handler_state.set_player_uuid(player_uuid);
        });
        self.login_success(username, player_uuid, protocol_version, client_connection);
    }

    fn login_success(&mut self, username: String, uuid: uuid::Uuid, protocol_version: i32, client_connection: ConnectionWrapper) {
        if self.players.contains_key(&uuid) {
            // TODO: Kick duplicqted playeers
            error!(%uuid, %username, "Duplicate player joining");
        }
        info!(name = %username, "Player joined the game!");
        let (spawn_pos, spawn_look) = (FalconConfig::global().spawn_pos(), FalconConfig::global().spawn_look());
        let player = Player::new(username, uuid, self.entity_id_count, spawn_pos, spawn_look, protocol_version, client_connection);
        self.entity_id_count += 1;

        let player = self.players.entry(uuid).or_insert(player);
        let join_game_spec = JoinGameSpec::new(player, Difficulty::Peaceful, FalconConfig::global().max_players() as u8, String::from("customized"), FalconConfig::global().max_view_distance() as i32, false);
        player.connection().build_send_packet(join_game_spec, |p, c| falcon_send::send_join_game(p, c));
        let player_abilities = PlayerAbilitiesSpec::new(player, 0.05, 0.1);
        player.connection().build_send_packet(player_abilities, |p, c| falcon_send::send_player_abilities(p, c));
        self.world.send_chunks_for_player(player, CHUNK_FN, CHUNK_AIR_FN);
        let position_look = PositionAndLookSpec::new(player, 0, 1);
        player.connection().build_send_packet(position_look, |p, c| falcon_send::send_position_look(p, c));
    }

    fn player_leave(&mut self, uuid: Uuid) {
        let player = self.players.remove(&uuid);
        if let Some(player) = player {
            info!(%uuid, name = player.username(), "Player disconnected!");
        }
    }

    /// Signals through the update of the player's position and look
    fn player_update_pos_look(
        &mut self,
        uuid: Uuid,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        _on_ground: Option<bool>,
    ) {
        // TODO: make more fancy
        // TODO: fire event
        if let Entry::Occupied(mut entry) = self.players.entry(uuid) {
            let player = entry.get_mut();
            let look_angles = player.look_angles_mut();
            yaw.map(|e| look_angles.set_yaw(e));
            pitch.map(|e| look_angles.set_pitch(e));

            let position = player.position_mut();
            let (old_chunk_x, old_chunk_z) = (position.chunk_x(), position.chunk_z());
            x.map(|x| position.set_x(x));
            y.map(|y| position.set_y(y));
            z.map(|z| position.set_z(z));
            let (chunk_x, chunk_z) = (position.chunk_x(), position.chunk_z());
            if chunk_x != old_chunk_x || chunk_z != old_chunk_z {
                self.world.update_player_pos(player, old_chunk_x, old_chunk_z, chunk_x, chunk_z, CHUNK_FN, CHUNK_AIR_FN, UNLOAD_FN);
            }
        }
    }

    fn player_update_view_distance(&mut self, uuid: Uuid, view_distance: u8) {
        if let Entry::Occupied(mut entry) = self.players.entry(uuid) {
            let player = entry.get_mut();
            self.world.update_view_distance(player, view_distance, CHUNK_FN, CHUNK_AIR_FN, UNLOAD_FN);
            player.set_view_distance(view_distance);
        }
    }
}

const CHUNK_FN: fn(&mut dyn MinecraftPlayer, &Chunk) = |player: &mut dyn MinecraftPlayer, chunk: &Chunk| {
    let packet = ChunkDataSpec::new(chunk, player.protocol_version());
    player.connection().build_send_packet(packet, |p, c| falcon_send::send_chunk_data(p, c));
};
const CHUNK_AIR_FN: fn(&mut dyn MinecraftPlayer, i32, i32) = |player: &mut dyn MinecraftPlayer, x: i32, z: i32| {
    let packet = ChunkDataSpec::empty(x, z);
    player.connection().build_send_packet(packet, |p, c| falcon_send::send_chunk_data(p, c));
};
const UNLOAD_FN: fn(&mut dyn MinecraftPlayer, i32, i32) = |player: &mut dyn MinecraftPlayer, x: i32, z: i32| {
    player.connection().build_send_packet((x, z), |p, c| falcon_send::send_unload_chunk(p, c));
};
