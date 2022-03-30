pub use wrapper::ConnectionLogic;

use mc_chat::ChatComponent;

use falcon_core::network::connection::ClientConnection;
use falcon_core::network::ConnectionState;

mod wrapper;

pub fn disconnect(connection: &mut ClientConnection, reason: ChatComponent) {
    match connection.handler_state().connection_state() {
        ConnectionState::Play => falcon_send::send_play_disconnect(reason, connection),
        _ => falcon_send::send_login_disconnect(reason, connection),
    }
    connection.handler_state_mut().set_connection_state(ConnectionState::Disconnected);
}