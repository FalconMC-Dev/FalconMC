use mc_chat::ChatComponent;

use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic};
use falcon_core::network::ConnectionState;

#[derive(Debug)]
pub struct FalconConnection<D: ConnectionDriver> {
    driver: D,
}

impl<D: ConnectionDriver> FalconConnection<D> {
    pub fn reset_keep_alive(&mut self) {
        self.driver.reset_timeout();
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

