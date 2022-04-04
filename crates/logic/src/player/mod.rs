pub use wrapper::PlayerLogic;

use mc_chat::ChatComponent;

use falcon_core::player::Player;
use crate::ConnectionLogic;

mod wrapper;

impl PlayerLogic for Player {
    fn disconnect(&mut self, reason: ChatComponent) {
        self.connection().disconnect(reason);
    }

    #[tracing::instrument(skip(self))]
    fn send_keep_alive(&self) {
        let elapsed = self.time.elapsed().as_secs();
        self.connection().execute(move |connection| {
            connection.handler_state_mut().set_last_keep_alive(elapsed);
            falcon_send::send_keep_alive(elapsed as i64, connection);
        });
    }
}
