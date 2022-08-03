use ahash::AHashMap;
use falcon_core::ShutdownHandle;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

use falcon_core::network::connection::ConnectionDriver;
use crate::player::FalconPlayer;
use crate::world::FalconWorld;

mod status;
mod login;
mod play;

pub struct FalconServer<D: ConnectionDriver> {
    shutdown: ShutdownHandle,
    should_stop: bool,
    console_rx: UnboundedReceiver<String>,
    eid_count: i32,
    players: AHashMap<Uuid, FalconPlayer<D>>,
    world: FalconWorld,
}

impl<D: ConnectionDriver> FalconServer<D> {
    pub fn new(shutdown: ShutdownHandle, console_rx: UnboundedReceiver<String>, world: FalconWorld) -> Self {
        Self {
            shutdown,
            should_stop: false,
            console_rx,
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

