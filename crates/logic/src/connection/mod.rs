use mc_chat::ChatComponent;

use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic, ConnectionWrapper};
use falcon_core::network::ConnectionState;

use crate::server::ServerWrapper;

#[derive(Debug)]
pub struct FalconConnection<D: ConnectionDriver> {
    driver: D,
    server: ServerWrapper<D>,
    wrapper: ConnectionWrapper<D, Self>,
}

impl<D: ConnectionDriver> FalconConnection<D> {
    pub fn new(driver: D, wrapper: ConnectionWrapper<D, Self>, server: ServerWrapper<D>) -> Self {
        Self {
            driver,
            server,
            wrapper,
        }
    }

    pub fn reset_keep_alive(&mut self) {
        self.driver.reset_timeout();
    }

    pub fn server(&self) -> &ServerWrapper<D> {
        &self.server
    }

    pub fn wrapper(&self) -> ConnectionWrapper<D, Self> {
        self.wrapper.clone()
    }
}

impl<D: ConnectionDriver> ConnectionLogic<D> for FalconConnection<D> {
    fn disconnect(&mut self, reason: ChatComponent) {
        match self.driver.handler_state().connection_state() {
            ConnectionState::Play => falcon_send::send_play_disconnect(reason, self),
            _ => falcon_send::send_login_disconnect(reason, self),
        }
        self.driver.handler_state_mut().set_connection_state(ConnectionState::Disconnected);
    }

    fn driver(&self) -> &D {
        &self.driver
    }

    fn driver_mut(&mut self) -> &mut D {
        &mut self.driver
    }
}

