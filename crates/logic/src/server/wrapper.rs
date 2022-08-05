use falcon_core::network::connection::{ConnectionDriver, ConnectionWrapper};
use falcon_core::player::data::Position;
use ignore_result::Ignore;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::{FalconServer, FalconConnection};

use super::ServerTask;

#[derive(Debug)]
pub struct ServerWrapper<D: ConnectionDriver> {
    link: UnboundedSender<ServerTask<D>>,
}

impl<D: ConnectionDriver> ServerWrapper<D> {
    pub fn new(link: UnboundedSender<ServerTask<D>>) -> Self {
        Self {
            link,
        }
    }

    /// Do not pass a `Box` to this function.
    pub fn execute_sync<T>(&self, task: T)
        where
            T: FnOnce(&mut FalconServer<D>) + Send + Sync + 'static,
    {
        self.link.send(ServerTask::Sync(Box::new(task))).ignore();
    }
}

impl<D: ConnectionDriver + 'static> ServerWrapper<D> {
    pub fn request_status(&self, protocol: i32, connection: ConnectionWrapper<D, FalconConnection<D>>) {
        self.execute_sync(move |server| {
            server.request_status(protocol, connection);
        })
    }

    pub fn player_login(&self, username: String, protocol: i32, connection: ConnectionWrapper<D, FalconConnection<D>>) {
        self.execute_sync(move |server| {
            server.player_login(username, protocol, connection);
        })
    }

    pub fn player_update_pos_look(&self, uuid: Uuid, pos: Option<Position>, facing: Option<(f32, f32)>, on_ground: bool) {
        self.execute_sync(move |server| {
            server.player_update_pos_look(uuid, pos, facing, on_ground);
        })
    }

    pub fn player_update_view_distance(&self, uuid: Uuid, view_distance: u8) {
        self.execute_sync(move |server| {
            server.player_update_view_distance(uuid, view_distance);
        })
    }
}

impl<D: ConnectionDriver> Clone for ServerWrapper<D> {
    fn clone(&self) -> Self {
        Self { link: self.link.clone() }
    }
}


