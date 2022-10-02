use falcon_packet_core::WriteError;
use ignore_result::Ignore;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use crate::FalconConnection;

use super::{writer::SocketWrite, ConnectionTask};

#[derive(Debug)]
pub struct ConnectionWrapper {
    link: UnboundedSender<ConnectionTask>,
}

impl ConnectionWrapper {
    pub fn new(link: UnboundedSender<ConnectionTask>) -> Self {
        ConnectionWrapper { link }
    }

    pub fn reset_keep_alive(&self) {
        self.link
            .send(ConnectionTask::Sync(Box::new(|connection| {
                connection.reset_keep_alive();
            })))
            .ignore();
    }

    pub fn send_packet<T, F>(&self, packet: T, write_fn: F)
    where
        T: Send + Sync + 'static,
        F: FnOnce(T, &mut SocketWrite, i32) -> Result<bool, WriteError> + Send + Sync + 'static,
    {
        self.link
            .send(ConnectionTask::Sync(Box::new(move |connection| {
                if let Err(err) = connection.send_packet(packet, write_fn) {
                    error!("Error when sending packet: {}", err);
                }
            })))
            .ignore()
    }

    /// Do not pass a `Box` to this function.
    pub fn execute_sync<T>(&self, task: T)
    where
        T: FnOnce(&mut FalconConnection) + Send + Sync + 'static,
    {
        self.link
            .send(ConnectionTask::Sync(Box::new(task)))
            .ignore();
    }
}

impl Clone for ConnectionWrapper {
    fn clone(&self) -> Self {
        ConnectionWrapper {
            link: self.link.clone(),
        }
    }
}
