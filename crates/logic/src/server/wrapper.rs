use std::convert::Infallible;
use std::error::Error;

use anyhow::Result;
use falcon_core::player::data::Position;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::{ServerTask, SyncServerTask};
use crate::connection::ConnectionWrapper;
use crate::FalconServer;

#[derive(Debug)]
pub struct ServerWrapper {
    link: UnboundedSender<ServerTask>,
}

impl ServerWrapper {
    pub fn new(link: UnboundedSender<ServerTask>) -> Self { Self { link } }

    /// Do not pass a `Box` to this function.
    #[inline]
    pub fn send<T>(&self, task: T)
    where
        T: SyncServerTask + 'static,
    {
        // SAFE: if this channel returns an error, the server will have shut down
        // already.
        self.link.send(ServerTask::Sync(Box::new(task))).ok();
    }

    /// Do not pass a `Box` to this function.
    pub fn execute<F, E>(&self, task: F)
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce(&mut FalconServer) -> Result<(), E> + Send + Sync + 'static,
    {
        self.send(task);
    }
}

impl ServerWrapper {
    pub fn request_status(&self, protocol: i32, connection: ConnectionWrapper) {
        self.execute(move |server| {
            server.request_status(protocol, connection);
            Ok::<(), Infallible>(())
        });
    }

    pub fn player_login(&self, username: String, protocol: i32, connection: ConnectionWrapper) {
        self.execute(move |server| {
            server.player_login(username, protocol, connection);
            Ok::<(), Infallible>(())
        });
    }

    pub fn player_update_pos_look(&self, uuid: Uuid, pos: Option<Position>, facing: Option<(f32, f32)>, on_ground: bool) {
        self.execute(move |server| {
            server.player_update_pos_look(uuid, pos, facing, on_ground);
            Ok::<(), Infallible>(())
        });
    }

    pub fn player_update_view_distance(&self, uuid: Uuid, view_distance: u8) {
        self.execute(move |server| {
            server.player_update_view_distance(uuid, view_distance);
            Ok::<(), Infallible>(())
        });
    }

    pub fn player_leave(&self, uuid: Uuid) {
        self.execute(move |server| {
            server.player_leave(uuid);
            Ok::<(), Infallible>(())
        });
    }
}

impl Clone for ServerWrapper {
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}
