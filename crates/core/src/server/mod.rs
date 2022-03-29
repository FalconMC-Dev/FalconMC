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
    entity_id_count: i32,
    players: AHashMap<Uuid, Player>,
    // world
    world: World,
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
            world
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

    pub fn player_map(&mut self) -> &mut AHashMap<Uuid, Player> {
        &mut self.players
    }
}

#[derive(Debug)]
pub struct ServerWrapper {
    link: UnboundedSender<Box<McTask>>,
}

/*impl ServerWrapper {
    fn request_status(&self, protocol_id: i32, connection: ConnectionWrapper) {
        self.link.send(Box::new(move |server| {
            server.request_status(protocol_id, connection);
        })).ignore();
    }

    fn player_login(&mut self, username: String, protocol_version: i32, client_connection: ConnectionWrapper) {
        self.link.send(Box::new(move |server| {
            server.player_login(username, protocol_version, client_connection);
        })).ignore();
    }

    fn login_success(&mut self, username: String, uuid: Uuid, protocol_version: i32, client_connection: ConnectionWrapper) {
        self.link.send(Box::new(move |server| {
            server.login_success(username, uuid, protocol_version, client_connection);
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
}*/

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
}
