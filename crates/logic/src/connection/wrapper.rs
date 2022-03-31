use ignore_result::Ignore;
use mc_chat::ChatComponent;
use falcon_core::network::connection::ConnectionWrapper;

pub trait ConnectionLogic {
    fn disconnect(&self, reason: ChatComponent);
}

impl ConnectionLogic for ConnectionWrapper {
    fn disconnect(&self, reason: ChatComponent) {
        self.link().send(Box::new(move |connection| {
            super::disconnect(connection, reason);
        })).ignore();
    }
}