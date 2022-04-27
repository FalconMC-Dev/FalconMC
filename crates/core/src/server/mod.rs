//! Part of the Public API of FalconMC

use ahash::AHashMap;
use ignore_result::Ignore;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

use crate::player::Player;
use crate::ShutdownHandle;
use crate::world::World;

pub mod config;
pub mod data;

pub type McTask = dyn FnOnce(&mut MainServer) + Send + Sync;

pub struct MainServer {
    // threads
    shutdown_handle: ShutdownHandle,
    pub should_stop: bool,
    pub console_rx: UnboundedReceiver<String>,
    pub server_rx: UnboundedReceiver<Box<McTask>>,
    // players
    pub entity_id_count: i32,
    pub players: AHashMap<Uuid, Player>,
    // world
    pub world: World,
}

impl MainServer {
    pub fn new(
        shutdown_handle: ShutdownHandle,
        console_rx: UnboundedReceiver<String>,
        server_rx: UnboundedReceiver<Box<McTask>>,
        players: AHashMap<Uuid, Player>,
        world: World,
    ) -> Self {
        debug!("Player size: {}", std::mem::size_of::<Player>());
        MainServer {
            shutdown_handle,
            should_stop: false,
            console_rx,
            server_rx,
            entity_id_count: 0,
            players,
            world,
        }
    }

    pub fn shutdown_handle(&mut self) -> &mut ShutdownHandle {
        &mut self.shutdown_handle
    }

    pub fn online_count(&self) -> i32 {
        self.players.len() as i32
    }

    pub fn player(&self, uuid: Uuid) -> Option<&Player> {
        self.players.get(&uuid)
    }

    pub fn player_mut(&mut self, uuid: Uuid) -> Option<&mut Player> {
        self.players.get_mut(&uuid)
    }

    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }
}

#[derive(Debug)]
pub struct ServerWrapper {
    link: UnboundedSender<Box<McTask>>,
}

impl ServerWrapper {
    pub fn new(link: UnboundedSender<Box<McTask>>) -> Self {
        ServerWrapper {
            link,
        }
    }

    pub fn execute<T>(&self, task: T)
    where
        T: FnOnce(&mut MainServer) + Send + Sync + 'static,
    {
        self.link.send(Box::new(task)).ignore()
    }

    pub fn link(&self) -> &UnboundedSender<Box<McTask>> {
        &self.link
    }
}
