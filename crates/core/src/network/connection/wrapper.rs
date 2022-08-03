use std::fmt::Debug;
use ignore_result::Ignore;
use tokio::sync::mpsc::UnboundedSender;

use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic, ConnectionTask};

#[derive(Debug)]
pub struct ConnectionWrapper<D: ConnectionDriver, L: ConnectionLogic<D>> {
    link: UnboundedSender<ConnectionTask<D, L>>,
}

impl<D: ConnectionDriver, L: ConnectionLogic<D>> ConnectionWrapper<D, L> {
    pub fn new(link: UnboundedSender<ConnectionTask<D, L>>) -> Self {
        ConnectionWrapper {
            link,
        }
    }

    pub fn reset_keep_alive(&self) {
        self.link.send(ConnectionTask::Sync(Box::new(|conn| {
            conn.driver_mut().reset_timeout();
        }))).ignore();
    }
    
    pub fn build_send_packet<T>(&self, packet: T, func: fn(T, &mut L))
    where
        T: Sync + Send + 'static,
        L: 'static,
    {
        self.link.send(ConnectionTask::Sync(Box::new(move |connection| {
            func(packet, connection)
        }))).ignore();
    }

    /// Do not pass a `Box` to this function.
    pub fn execute_sync<T>(&self, task: T)
        where
            T: FnOnce(&mut L) + Send + Sync + 'static,
    {
        self.link.send(ConnectionTask::Sync(Box::new(task))).ignore();
    }
}

impl<D: ConnectionDriver, L: ConnectionLogic<D>> Clone for ConnectionWrapper<D, L> {
    fn clone(&self) -> Self {
        ConnectionWrapper {
            link: self.link.clone(),
        }
    }
}
