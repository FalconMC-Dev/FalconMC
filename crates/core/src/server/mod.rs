//! Part of the Public API of FalconMC

use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::network::connection::ConnectionTask;

pub mod config;

pub type McTask = dyn FnOnce(&mut dyn MinecraftServer) -> () + Send + Sync;

pub trait MinecraftServer {
    fn player_join(&mut self, username: String, uuid: Uuid, protocol_version: i32, client_connection: UnboundedSender<Box<ConnectionTask>>);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Peaceful = 0,
    Easy,
    Normal,
    Hard
}