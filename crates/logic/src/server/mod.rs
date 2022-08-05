use std::future::Future;
use std::pin::Pin;

use ahash::AHashMap;
use falcon_core::ShutdownHandle;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

use falcon_core::network::connection::ConnectionDriver;
use crate::player::FalconPlayer;
use crate::world::FalconWorld;

pub use wrapper::ServerWrapper;

mod status;
mod login;
mod play;
mod wrapper;

pub type SyncServerTask<D> = dyn FnOnce(&mut FalconServer<D>) + Send + Sync;
pub type AsyncServerTask<D> = dyn (FnOnce(&mut FalconServer<D>) -> Pin<Box<dyn Future<Output=()>>>) + Send + Sync;

pub enum ServerTask<D: ConnectionDriver> {
    Sync(Box<SyncServerTask<D>>),
    Async(Box<AsyncServerTask<D>>),
}

pub struct FalconServer<D: ConnectionDriver> {
    shutdown: ShutdownHandle,
    should_stop: bool,
    console_rx: UnboundedReceiver<String>,
    receiver: UnboundedReceiver<ServerTask<D>>,
    eid_count: i32,
    players: AHashMap<Uuid, FalconPlayer<D>>,
    world: FalconWorld,
}

impl<D: ConnectionDriver> FalconServer<D> {
    pub fn new(shutdown: ShutdownHandle, console_rx: UnboundedReceiver<String>, receiver: UnboundedReceiver<ServerTask<D>>, world: FalconWorld) -> Self {
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

    pub fn player(&self, uuid: Uuid) -> Option<&FalconPlayer<D>> {
        self.players.get(&uuid)
    }

    pub fn player_mut(&mut self, uuid: Uuid) -> Option<&mut FalconPlayer<D>> {
        self.players.get_mut(&uuid)
    }

    pub fn world(&mut self) -> &mut FalconWorld {
        &mut self.world
    }
}

