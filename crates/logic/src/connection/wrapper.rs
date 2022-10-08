use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;

use anyhow::Result;
use falcon_packet_core::WriteError;
use tokio::sync::mpsc::UnboundedSender;

use super::writer::SocketWrite;
use super::{ConnectionTask, SyncConnectionTask};
use crate::FalconConnection;

#[derive(Debug)]
pub struct ConnectionWrapper {
    link: UnboundedSender<ConnectionTask>,
}

impl ConnectionWrapper {
    pub fn new(link: UnboundedSender<ConnectionTask>) -> Self { ConnectionWrapper { link } }

    pub fn reset_keep_alive(&self) {
        self.execute(|connection| {
            connection.reset_keep_alive();
            Ok::<(), Infallible>(())
        })
    }

    pub fn send_packet<T, F>(&self, packet: T, write_fn: F)
    where
        T: Send + Sync + 'static,
        F: FnOnce(T, &mut SocketWrite, i32) -> Result<bool, WriteError> + Send + Sync + 'static,
    {
        self.execute(move |connection| -> Result<(), WriteError> {
            connection.send_packet(packet, write_fn)?;
            Ok(())
        });
    }

    /// Do not pass a `Box` to this function.
    #[inline]
    pub fn send<T>(&self, task: T)
    where
        T: SyncConnectionTask + 'static,
    {
        // SAFE: if this channel returns an error, then the client will have
        // disconnected already.
        self.link.send(ConnectionTask::Sync(Box::new(task))).ok();
    }

    /// Do not pass a `Box` to this function.
    pub fn execute<F, E>(&self, task: F)
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce(&mut FalconConnection) -> Result<(), E> + Send + Sync + 'static,
    {
        self.send(task);
    }
}

impl Clone for ConnectionWrapper {
    fn clone(&self) -> Self {
        ConnectionWrapper {
            link: self.link.clone(),
        }
    }
}
