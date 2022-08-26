use std::future::Future;
use std::pin::Pin;

use ahash::AHashMap;
use falcon_core::ShutdownHandle;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

use crate::player::FalconPlayer;
use crate::world::FalconWorld;

pub use wrapper::ServerWrapper;

mod network;
mod tick;
mod wrapper;

pub type SyncServerTask = dyn FnOnce(&mut FalconServer) + Send + Sync;
pub type AsyncServerTask = dyn (FnOnce(&mut FalconServer) -> Pin<Box<dyn Future<Output=()>>>) + Send + Sync;

pub enum ServerTask {
    Sync(Box<SyncServerTask>),
    Async(Box<AsyncServerTask>),
}

pub struct FalconServer {
    shutdown: ShutdownHandle,
    should_stop: bool,
    console_rx: UnboundedReceiver<String>,
    receiver: UnboundedReceiver<ServerTask>,
    eid_count: i32,
    players: AHashMap<Uuid, FalconPlayer>,
    world: Option<FalconWorld>,
}

impl FalconServer {
    pub fn new(
        shutdown: ShutdownHandle,
        console_rx: UnboundedReceiver<String>,
        receiver: UnboundedReceiver<ServerTask>,
        world: Option<FalconWorld>,
    ) -> Self {
        Self {
            shutdown,
            should_stop: false,
            console_rx,
            receiver,
            eid_count: 0,
            players: AHashMap::new(),
            world,
        }
    }

    pub fn shutdown_handle(&mut self) -> &mut ShutdownHandle {
        &mut self.shutdown
    }

    pub fn online_count(&self) -> usize {
        self.players.len()
    }

    pub fn player(&self, uuid: Uuid) -> Option<&FalconPlayer> {
        self.players.get(&uuid)
    }

    pub fn player_mut(&mut self, uuid: Uuid) -> Option<&mut FalconPlayer> {
        self.players.get_mut(&uuid)
    }

    pub fn world(&mut self) -> &mut Option<FalconWorld> {
        &mut self.world
    }
}
