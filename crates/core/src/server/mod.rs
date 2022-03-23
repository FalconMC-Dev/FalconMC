//! Part of the Public API of FalconMC

use ignore_result::Ignore;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use serde::Serialize;
use falcon_core::network::packet::TaskScheduleError;

use crate::network::connection::ConnectionWrapper;
use crate::network::packet::TaskScheduleResult;
use crate::player::MinecraftPlayer;

pub mod config;

pub type McTask = dyn FnOnce(&mut dyn MinecraftServer) + Send + Sync;

pub trait ServerData {
    /// Returns the number of players currently on the server.
    fn online_count(&self) -> i32;

    /// Returns the [`MinecraftPlayer`] associated with the given `uuid`
    /// if present on the server, otherwise [`None`].
    fn player(&self, uuid: Uuid) -> Option<&dyn MinecraftPlayer>;

    /// Same functionality as [`ServerData::player`] but returns a
    /// mutable reference to the requested player.
    fn player_mut(&mut self, uuid: Uuid) -> Option<&mut dyn MinecraftPlayer>;
}

pub trait ServerActor {
    /// Called when a player executes a successful login,
    /// i.e. when a player should spawn in the world.
    fn player_join(&mut self, username: String, uuid: Uuid, protocol_version: i32, client_connection: ConnectionWrapper);

    /// Called when a player loses connection with the server,
    /// i.e. when a connection breaks or the player leaves the game.
    fn player_leave(&mut self, uuid: Uuid);

    /// Called when a player's position should be updated,
    /// i.e. when a player moves, looks around or jumps.
    ///
    /// Leaving out specific fields will be treated as "*no update sent*".
    #[allow(clippy::too_many_arguments)]
    fn player_update_pos_look(&mut self, uuid: Uuid, x: Option<f64>, y: Option<f64>, z: Option<f64>, yaw: Option<f32>, pitch: Option<f32>, on_ground: Option<bool>);

    /// Called when a player changes their render distance, this
    /// should change the amount of chunks being sent to the player.
    fn player_update_view_distance(&mut self, uuid: Uuid, view_distance: u8);
}

pub trait MinecraftServer: ServerData + ServerActor {}

impl<T: ServerData + ServerActor> MinecraftServer for T {}

#[derive(Debug)]
pub struct ServerWrapper {
    link: UnboundedSender<Box<McTask>>,
}

impl ServerActor for ServerWrapper {
    fn player_join(&mut self, username: String, uuid: Uuid, protocol_version: i32, client_connection: ConnectionWrapper) {
        self.link.send(Box::new(move |server| {
            server.player_join(username, uuid, protocol_version, client_connection);
        })).ignore();
    }

    fn player_leave(&mut self, uuid: Uuid) {
        self.link.send(Box::new(move |server| {
            server.player_leave(uuid);
        })).ignore();
    }

    fn player_update_pos_look(&mut self, uuid: Uuid, x: Option<f64>, y: Option<f64>, z: Option<f64>, yaw: Option<f32>, pitch: Option<f32>, on_ground: Option<bool>) {
        self.link.send(Box::new(move |server| {
            server.player_update_pos_look(uuid, x, y, z, yaw, pitch, on_ground);
        })).ignore()
    }

    fn player_update_view_distance(&mut self, uuid: Uuid, view_distance: u8) {
        self.link.send(Box::new(move |server| {
            server.player_update_view_distance(uuid, view_distance);
        })).ignore();
    }
}

impl ServerWrapper {
    pub fn new(link: UnboundedSender<Box<McTask>>) -> Self {
        ServerWrapper {
            link,
        }
    }

    pub fn execute<T>(&self, task: T) -> TaskScheduleResult
    where
        T: FnOnce(&mut dyn MinecraftServer) + Send + Sync + 'static,
    {
        self.link.send(Box::new(task))
            .map_err(|_| TaskScheduleError::ThreadSendError)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Peaceful = 0,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Serialize, new)]
pub struct ServerVersion {
    pub name: String,
    pub protocol: i32,
}
