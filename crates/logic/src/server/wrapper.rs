use ignore_result::Ignore;
use uuid::Uuid;
use falcon_core::network::connection::ConnectionWrapper;
use falcon_core::server::ServerWrapper;

pub trait ServerLogic {
    fn request_status(&self, protocol: i32, connection: ConnectionWrapper);

    fn player_login(&self, username: String, protocol: i32, connection: ConnectionWrapper);

    fn login_success(&self, username: String, uuid: Uuid, protocol: i32, connection: ConnectionWrapper);

    fn player_leave(&self, uuid: Uuid);

    #[allow(clippy::too_many_arguments)]
    fn player_update_pos_look(&self, uuid: Uuid, x: Option<f64>, y: Option<f64>, z: Option<f64>, yaw: Option<f32>, pitch: Option<f32>, on_ground: bool);

    fn player_update_view_distance(&self, uuid: Uuid, view_distance: u8);
}

impl ServerLogic for ServerWrapper {
    fn request_status(&self, protocol: i32, connection: ConnectionWrapper) {
        self.link().send(Box::new(move |server| {
            super::request_status(server, protocol, connection);
        })).ignore();
    }

    fn player_login(&self, username: String, protocol_version: i32, client_connection: ConnectionWrapper) {
        self.link().send(Box::new(move |server| {
            super::player_login(server, username, protocol_version, client_connection);
        })).ignore();
    }

    fn login_success(&self, username: String, uuid: Uuid, protocol_version: i32, client_connection: ConnectionWrapper) {
        self.link().send(Box::new(move |server| {
            super::login_success(server, username, uuid, protocol_version, client_connection);
        })).ignore();
    }

    fn player_leave(&self, uuid: Uuid) {
        self.link().send(Box::new(move |server| {
            super::player_leave(server, uuid);
        })).ignore();
    }

    fn player_update_pos_look(&self, uuid: Uuid, x: Option<f64>, y: Option<f64>, z: Option<f64>, yaw: Option<f32>, pitch: Option<f32>, on_ground: bool) {
        self.link().send(Box::new(move |server| {
            super::player_update_pos_look(server, uuid, x, y, z, yaw, pitch, on_ground);
        })).ignore()
    }

    fn player_update_view_distance(&self, uuid: Uuid, view_distance: u8) {
        self.link().send(Box::new(move |server| {
            super::player_update_view_distance(server, uuid, view_distance);
        })).ignore();
    }
}