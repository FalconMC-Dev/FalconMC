use anyhow::Result;
use falcon_core::player::data::Position;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::ServerTask;
use crate::connection::ConnectionWrapper;
use crate::FalconServer;

#[derive(Debug)]
pub struct ServerWrapper {
    link: UnboundedSender<ServerTask>,
}

impl ServerWrapper {
    pub fn new(link: UnboundedSender<ServerTask>) -> Self { Self { link } }

    /// Do not pass a `Box` to this function.
    pub fn execute_sync<T>(&self, task: T)
    where
        T: FnOnce(&mut FalconServer) -> Result<()> + Send + Sync + 'static,
    {
        self.link.send(ServerTask::Sync(Box::new(task))).ok();
    }
}

impl ServerWrapper {
    pub fn request_status(&self, protocol: i32, connection: ConnectionWrapper) {
        self.execute_sync(move |server| {
            Ok(server.request_status(protocol, connection))
        })
    }

    pub fn player_login(&self, username: String, protocol: i32, connection: ConnectionWrapper) {
        self.execute_sync(move |server| {
            Ok(server.player_login(username, protocol, connection))
        })
    }

    pub fn player_update_pos_look(&self, uuid: Uuid, pos: Option<Position>, facing: Option<(f32, f32)>, on_ground: bool) {
        self.execute_sync(move |server| {
            Ok(server.player_update_pos_look(uuid, pos, facing, on_ground))
        })
    }

    pub fn player_update_view_distance(&self, uuid: Uuid, view_distance: u8) {
        self.execute_sync(move |server| {
            Ok(server.player_update_view_distance(uuid, view_distance))
        })
    }

    pub fn player_leave(&self, uuid: Uuid) {
        self.execute_sync(move |server| {
            Ok(server.player_leave(uuid))
        })
    }
}

impl Clone for ServerWrapper {
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}
