use falcon_core::network::connection::{ConnectionDriver, ConnectionWrapper};
use falcon_core::server::config::FalconConfig;
use falcon_core::server::data::ServerVersion;
use falcon_send::specs::status::{PlayerData, StatusResponseSpec};

use crate::connection::FalconConnection;

use super::FalconServer;

impl<D: ConnectionDriver + 'static> FalconServer<D> {
    pub fn request_status(&self, protocol: i32, connection: ConnectionWrapper<D, FalconConnection<D>>) {
        let version = ServerVersion::new(String::from("1.13-1.17.1"), protocol);
        let player_data = PlayerData::new(FalconConfig::global().max_players(), self.online_count() as i32);
        let description = String::from(FalconConfig::global().description());
        connection.build_send_packet(
            StatusResponseSpec::new(version, player_data, description),
            falcon_send::send_status_response
        );
    }
}
