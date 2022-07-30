use std::fmt::Debug;
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
}

impl<D: ConnectionDriver, L: ConnectionLogic<D>> Clone for ConnectionWrapper<D, L> {
    fn clone(&self) -> Self {
        ConnectionWrapper {
            link: self.link.clone(),
        }
    }
}