//! Part of the Public API of FalconMC

use std::fmt::Debug;
use ahash::AHashMap;
use ignore_result::Ignore;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;
use falcon_core::network::connection::ConnectionLogic;
use crate::network::connection::ConnectionDriver;

use crate::player::Player;
use crate::ShutdownHandle;
use crate::world::World;

pub mod config;
pub mod data;

pub type McTask<D, L> = dyn FnOnce(&mut MainServer<D, L>) + Send + Sync;

pub struct MainServer<D: ConnectionDriver<L>, L: ConnectionLogic> {
    // threads
    shutdown_handle: ShutdownHandle,
    pub should_stop: bool,
    pub console_rx: UnboundedReceiver<String>,
    pub server_rx: UnboundedReceiver<Box<McTask<D, L>>>,
    // players
    pub entity_id_count: i32,
    pub players: AHashMap<Uuid, Player<D, L>>,
    // world
    pub world: World,
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> MainServer<D, L> {
    pub fn new(
        shutdown_handle: ShutdownHandle,
        console_rx: UnboundedReceiver<String>,
        server_rx: UnboundedReceiver<Box<McTask<D, L>>>,
        players: AHashMap<Uuid, Player<D, L>>,
        world: World,
    ) -> Self {
        debug!("Player size: {}", std::mem::size_of::<Player<D, L>>());
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

    pub fn player(&self, uuid: Uuid) -> Option<&Player<D, L>> {
        self.players.get(&uuid)
    }

    pub fn player_mut(&mut self, uuid: Uuid) -> Option<&mut Player<D, L>> {
        self.players.get_mut(&uuid)
    }

    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }
}

#[derive(Debug)]
pub struct ServerWrapper<D: ConnectionDriver<L>, L: ConnectionLogic> {
    link: UnboundedSender<Box<McTask<D, L>>>,
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> ServerWrapper<D, L> {
    pub fn new(link: UnboundedSender<Box<McTask<D, L>>>) -> Self {
        ServerWrapper {
            link,
        }
    }

    pub fn execute<T>(&self, task: T)
    where
        T: FnOnce(&mut MainServer<D, L>) + Send + Sync + 'static,
    {
        self.link.send(Box::new(task)).ignore()
    }

    pub fn link(&self) -> &UnboundedSender<Box<McTask<D, L>>> {
        &self.link
    }
}
