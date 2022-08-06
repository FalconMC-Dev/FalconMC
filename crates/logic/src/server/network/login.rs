use falcon_core::network::ConnectionState;
use falcon_core::network::connection::ConnectionLogic;
use falcon_core::server::config::FalconConfig;
use falcon_core::server::data::Difficulty;
use falcon_send::specs::login::LoginSuccessSpec;
use falcon_send::specs::play::{ServerDifficultySpec, PlayerAbilitiesSpec, PositionAndLookSpec};
use uuid::Uuid;

use crate::connection::ConnectionWrapper;
use crate::player::FalconPlayer;

use crate::server::FalconServer;

impl FalconServer {
    pub fn player_login(&mut self, username: String, protocol: i32, connection: ConnectionWrapper) {
        debug!(player_name = %username);
        // create correct uuids
        let player_uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, username.as_bytes());
        let username2 = username.clone();
        connection.execute_sync(move |connection| {
            falcon_send::send_login_success(LoginSuccessSpec::new(player_uuid, username2), connection);
            let handler_state = connection.handler_state_mut();
            handler_state.set_connection_state(ConnectionState::Play);
            handler_state.set_player_uuid(player_uuid);
        });
        self.login_success(username, player_uuid, protocol, connection);
    }

    pub fn login_success(&mut self, username: String, uuid: Uuid, protocol: i32, connection: ConnectionWrapper) {
        if self.players.contains_key(&uuid) {
            // TODO: Kick duplicqted playeers
            error!(%uuid, %username, "Duplicate player joining");
        }
        info!(name = %username, "Player joined the game!");
        let (spawn_pos, spawn_look) = (FalconConfig::global().spawn_pos(), FalconConfig::global().spawn_look());
        let player = FalconPlayer::new(username, uuid, self.eid_count, spawn_pos, spawn_look, protocol, connection);
        self.eid_count += 1;

        self.players.insert(uuid, player);
        if let Some(player) = self.players.get(&uuid) {
            let join_game_spec = player.join_spec(Difficulty::Peaceful, FalconConfig::global().max_players() as u8, String::from("customized"), 0, false, false);
            player.connection().build_send_packet(join_game_spec, falcon_send::send_join_game);
            let server_difficulty = ServerDifficultySpec::new(Difficulty::Peaceful, false);
            player.connection().build_send_packet(server_difficulty, falcon_send::send_server_difficulty);
            let player_abilities = PlayerAbilitiesSpec::new(player.ability_flags(), 0.05, 0.1);
            player.connection().build_send_packet(player_abilities, falcon_send::send_player_abilities);
            self.world.send_chunks_for_player(player);
            let position_look = PositionAndLookSpec::new(player.position(), player.look_angles(), 0, 1);
            player.connection().build_send_packet(position_look, falcon_send::send_position_look);
        }
    }
}
