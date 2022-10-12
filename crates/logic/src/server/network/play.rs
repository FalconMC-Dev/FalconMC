use falcon_core::player::data::Position;
use tracing::info;
use uuid::Uuid;

use crate::server::FalconServer;

impl FalconServer {
    pub fn player_leave(&mut self, uuid: Uuid) {
        if let Some(player) = self.players.remove(&uuid) {
            self.usernames.remove(player.username());
            info!(%uuid, username = player.username(), "Player disconnected!");
        }
    }

    pub fn player_update_pos_look(&mut self, uuid: Uuid, pos: Option<Position>, facing: Option<(f32, f32)>, _on_ground: bool) {
        let mut update_position = false;
        let mut update_viewpos = false;
        let (old_x, old_z, x, z) = match self.players.get_mut(&uuid) {
            Some(player) => {
                let look_angles = player.look_angles_mut();
                if let Some((yaw, pitch)) = facing {
                    look_angles.yaw = yaw;
                    look_angles.pitch = pitch;
                }
                let position = player.position_mut();
                let (old_chunk_x, old_chunk_z) = position.chunk_coords();
                if let Some(pos) = pos {
                    position.x = pos.x;
                    position.z = pos.z;
                    if pos.y as i32 != position.y as i32 {
                        update_viewpos = true;
                    }
                    position.y = pos.y;
                }

                let (chunk_x, chunk_z) = (position.chunk_x(), position.chunk_z());
                if chunk_x != old_chunk_x || chunk_z != old_chunk_z {
                    update_viewpos = true;
                    update_position = true;
                }
                (old_chunk_x, old_chunk_z, chunk_x, chunk_z)
            },
            None => (0, 0, 0, 0),
        };
        if update_position {
            self.world.update_player_pos(self.players.get(&uuid).unwrap(), old_x, old_z, x, z);
        }
        if update_viewpos {
            self.players
                .get(&uuid)
                .unwrap()
                .connection()
                .send_packet((x, z), falcon_send::write_update_viewpos);
        }
    }

    pub fn player_update_view_distance(&mut self, uuid: Uuid, view_distance: u8) {
        if let Some(player) = self.players.get_mut(&uuid) {
            self.world.update_view_distance(player, view_distance);
            player.set_view_distance(view_distance);
        }
    }
}
