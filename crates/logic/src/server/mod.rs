use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use ahash::AHashMap;
use anyhow::Result;
use falcon_core::ShutdownHandle;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;
pub use wrapper::ServerWrapper;

use crate::player::FalconPlayer;
use crate::world::FalconWorld;

mod network;
mod tick;
mod wrapper;

pub trait SyncServerTask: Send + Sync {
    fn run(self: Box<Self>, server: &mut FalconServer) -> Result<()>;
}

pub trait SyncFutServerTask: Send + Sync {
    fn run(self: Box<Self>, server: &mut FalconServer) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

pub enum ServerTask {
    Sync(Box<dyn SyncServerTask>),
    Async(Box<dyn SyncFutServerTask>),
}

pub struct FalconServer {
    shutdown: ShutdownHandle,
    should_stop: bool,
    console_rx: UnboundedReceiver<String>,
    receiver: UnboundedReceiver<ServerTask>,
    eid_count: i32,
    players: AHashMap<Uuid, FalconPlayer>,
    usernames: AHashMap<String, Uuid>,
    world: FalconWorld,
}

impl FalconServer {
    pub fn new(shutdown: ShutdownHandle, console_rx: UnboundedReceiver<String>, receiver: UnboundedReceiver<ServerTask>, world: FalconWorld) -> Self {
        Self {
            shutdown,
            should_stop: false,
            console_rx,
            receiver,
            eid_count: 0,
            players: AHashMap::new(),
            usernames: AHashMap::new(),
            world,
        }
    }

    pub fn shutdown_handle(&mut self) -> &mut ShutdownHandle { &mut self.shutdown }

    pub fn online_count(&self) -> usize { self.players.len() }

    pub fn player(&self, uuid: Uuid) -> Option<&FalconPlayer> { self.players.get(&uuid) }

    pub fn player_mut(&mut self, uuid: Uuid) -> Option<&mut FalconPlayer> { self.players.get_mut(&uuid) }

    pub fn player_by_username(&mut self, username: &String) -> Option<&FalconPlayer> { self.usernames.get(username).and_then(|x| self.players.get(x)) }

    pub fn player_by_username_mut(&mut self, username: &String) -> Option<&mut FalconPlayer> { self.usernames.get(username).and_then(|x| self.players.get_mut(x)) }

    pub fn world(&mut self) -> &mut FalconWorld { &mut self.world }
}

impl<F, E> SyncServerTask for F
where
    E: Error + Send + Sync + 'static,
    F: FnOnce(&mut FalconServer) -> Result<(), E> + Send + Sync,
{
    fn run(self: Box<Self>, server: &mut FalconServer) -> Result<()> { Ok(self(server)?) }
}

impl<F, E> SyncFutServerTask for F
where
    E: Error + Send + Sync + 'static,
    F: FnOnce(&mut FalconServer) -> Pin<Box<dyn Future<Output = Result<(), E>> + Send>> + Send + Sync + 'static,
{
    fn run(self: Box<F>, server: &mut FalconServer) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> { Box::pin(async { Ok(self(server).await?) }) }
}
