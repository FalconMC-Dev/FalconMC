//! Part of the Public API of FalconMC

use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use serde::Serialize;

use crate::network::connection::ConnectionTask;
use crate::player::MinecraftPlayer;

pub mod config;

pub type McTask = dyn FnOnce(&mut dyn MinecraftServer) + Send + Sync;

pub trait MinecraftServer {
    fn online_player_count(&self) -> i32;

    fn get_player(&self, uuid: Uuid) -> Option<&dyn MinecraftPlayer>;

    fn get_player_mut(&mut self, uuid: Uuid) -> Option<&mut dyn MinecraftPlayer>;

    fn player_join(
        &mut self,
        username: String,
        uuid: Uuid,
        protocol_version: i32,
        client_connection: UnboundedSender<Box<ConnectionTask>>,
    );

    fn player_leave(&mut self, uuid: Uuid);

    fn player_position_and_look(
        &mut self,
        uuid: Uuid,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    );
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
