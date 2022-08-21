use bytes::Bytes;
use falcon_core::network::connection::ConnectionLogic;
use ignore_result::Ignore;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;

use crate::FalconConnection;

use super::ConnectionTask;

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

    pub fn build_send_packet<T>(&self, packet: T, func: fn(T, &mut FalconConnection))
    where
        T: Sync + Send + 'static,
    {
        self.link
            .send(ConnectionTask::Sync(Box::new(move |connection| {
                func(packet, connection)
            })))
            .ignore();
    }

    pub fn send_batch<B, C>(&self, batch: Vec<B>, mut convert: C)
    where
        C: FnMut(B) -> Option<Bytes>,
    {
        let mut packets = Vec::with_capacity(batch.len());
        for item in batch {
            if let Some(data) = convert(item) {
                packets.push(data);
            }
        }
        self.execute_sync(move |connection| {
            for packet in packets {
                connection.send(packet);
            }
        })
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
