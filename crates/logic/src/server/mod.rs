use std::collections::hash_map::Entry;
use uuid::Uuid;
pub use wrapper::ServerLogic;

use falcon_core::network::connection::ConnectionWrapper;
use falcon_core::network::ConnectionState;
use falcon_core::player::Player;
use falcon_core::server::config::FalconConfig;
use falcon_core::server::data::{Difficulty, ServerVersion};
use falcon_core::server::MainServer;
use falcon_send::specs::login::LoginSuccessSpec;
use falcon_send::specs::play::{JoinGameSpec, PlayerAbilitiesSpec, PositionAndLookSpec, ServerDifficultySpec};
use falcon_send::specs::status::{PlayerData, StatusResponseSpec};
use crate::world::{send_chunks_for_player, update_player_pos, update_view_distance};

mod wrapper;

pub fn request_status(server: &MainServer, protocol: i32, connection: ConnectionWrapper) {
    let version = ServerVersion::new(String::from("1.13-1.17.1"), protocol);
    let player_data = PlayerData::new(FalconConfig::global().max_players(), server.online_count());
    let description = String::from(FalconConfig::global().description());
    connection.build_send_packet(StatusResponseSpec::new(version, player_data, description), falcon_send::send_status_response);
}

pub fn player_login(server: &mut MainServer, username: String, protocol: i32, connection: ConnectionWrapper) {
    debug!(player_name = %username);
    let player_uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, username.as_bytes());
    let username2 = username.clone();
    connection.execute(move |connection| {
        falcon_send::send_login_success(LoginSuccessSpec::new(player_uuid, username2), connection);
        let handler_state = connection.handler_state_mut();
        handler_state.set_connection_state(ConnectionState::Play);
        handler_state.set_player_uuid(player_uuid);
    });
    login_success(server, username, player_uuid, protocol, connection);
}

pub fn login_success(server: &mut MainServer, username: String, uuid: Uuid, protocol: i32, connection: ConnectionWrapper) {
    if server.players.contains_key(&uuid) {
        // TODO: Kick duplicqted playeers
        error!(%uuid, %username, "Duplicate player joining");
    }
    info!(name = %username, "Player joined the game!");
    let (spawn_pos, spawn_look) = (FalconConfig::global().spawn_pos(), FalconConfig::global().spawn_look());
    let player = Player::new(username, uuid, server.entity_id_count, spawn_pos, spawn_look, protocol, connection);
    server.entity_id_count += 1;

    server.players.insert(uuid, player);
    if let Entry::Occupied(entry) = server.players.entry(uuid) {
        let player = entry.get();
        let join_game_spec = JoinGameSpec::new(player, Difficulty::Peaceful, FalconConfig::global().max_players() as u8, String::from("customized"), 0, FalconConfig::global().max_view_distance() as i32, false, false);
        player.connection().build_send_packet(join_game_spec, falcon_send::send_join_game);
        let server_difficulty = ServerDifficultySpec::new(Difficulty::Peaceful, false);
        player.connection().build_send_packet(server_difficulty, falcon_send::send_server_difficulty);
        let player_abilities = PlayerAbilitiesSpec::new(player, 0.05, 0.1);
        player.connection().build_send_packet(player_abilities, falcon_send::send_player_abilities);
        send_chunks_for_player(&server.world, player);
        let position_look = PositionAndLookSpec::new(player, 0, 1);
        player.connection().build_send_packet(position_look, falcon_send::send_position_look);
    }
}

pub fn player_leave(server: &mut MainServer, uuid: Uuid) {
    let player = server.players.remove(&uuid);
    if let Some(player) = player {
        info!(%uuid, name = player.username(), "Player disconnected!");
    }
}

#[allow(clippy::option_map_unit_fn)]
#[allow(clippy::too_many_arguments)]
pub fn player_update_pos_look(server: &mut MainServer, uuid: Uuid, x: Option<f64>, y: Option<f64>, z: Option<f64>, yaw: Option<f32>, pitch: Option<f32>, _on_ground: bool) {
    // TODO: make more fancy
    // TODO: fire event
    if let Entry::Occupied(mut entry) = server.players.entry(uuid) {
        let player = entry.get_mut();
        let look_angles = player.look_angles_mut();
        yaw.map(|e| look_angles.set_yaw(e));
        pitch.map(|e| look_angles.set_pitch(e));

        let mut update_viewpos = false;
        let position = player.position_mut();
        let (old_chunk_x, old_chunk_z) = (position.chunk_x(), position.chunk_z());
        x.map(|x| position.set_x(x));
        if let Some(y) = y {
            if y as i32 != position.y() as i32 {
                update_viewpos = true;
            }
            position.set_y(y);
        }
        z.map(|z| position.set_z(z));
        let (chunk_x, chunk_z) = (position.chunk_x(), position.chunk_z());
        if chunk_x != old_chunk_x || chunk_z != old_chunk_z {
            update_viewpos = true;
            update_player_pos(&server.world, player, old_chunk_x, old_chunk_z, chunk_x, chunk_z);
        }
        if update_viewpos {
            player.connection().build_send_packet((chunk_x, chunk_z), falcon_send::send_update_viewpos);
        }
    }
}

pub fn player_update_view_distance(server: &mut MainServer, uuid: Uuid, view_distance: u8) {
    if let Entry::Occupied(mut entry) = server.players.entry(uuid) {
        let player = entry.get_mut();
        update_view_distance(&server.world, player, view_distance);
        player.set_view_distance(view_distance);
    }
}
