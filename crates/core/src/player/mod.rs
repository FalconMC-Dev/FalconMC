use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::network::buffer::PacketBufferWrite;

use crate::network::connection::ConnectionWrapper;
use crate::network::packet::PacketEncode;

pub trait MinecraftPlayer {
    /// identity methods
    fn username(&self) -> &str;

    fn uuid(&self) -> Uuid;

    /// game methods
    fn entity_id(&self) -> i32;

    fn game_mode(&self) -> GameMode;

    fn dimension(&self) -> i32;

    fn ability_flags(&self) -> PlayerAbilityFlags;

    fn position(&self) -> &Position;

    /// This function should only be used to internally update the player's position
    /// as a result of an incoming packet, this does not update the position
    /// on the client-side!!!
    fn position_mut(&mut self) -> &mut Position;

    fn look_angles(&self) -> &LookAngles;

    /// This function should only be used to internally update the player's look angles
    /// as a result of an incoming packet, this does not update the angles
    /// on the client-side!!!
    fn look_angles_mut(&mut self) -> &mut LookAngles;

    fn view_distance(&self) -> u8;

    fn set_view_distance(&mut self, distance: u8);

    /// connection methods
    fn protocol_version(&self) -> i32;

    fn disconnect(&mut self, reason: String);

    fn client_connection(&mut self) -> &mut ConnectionWrapper;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameMode {
    Survival = 0,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct PlayerAbilityFlags {
    invulnerable: bool,
    flying: bool,
    allow_flying: bool,
    instant_break: bool,
}

impl PlayerAbilityFlags {
    pub fn new(invulnerable: bool, flying: bool, allow_flying: bool, instant_break: bool) -> Self {
        PlayerAbilityFlags {
            invulnerable,
            flying,
            allow_flying,
            instant_break,
        }
    }
}

impl PacketEncode for PlayerAbilityFlags {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        let mut byte = 0u8;
        if self.invulnerable {
            byte |= 1;
        }
        if self.flying {
            byte |= 1 << 1;
        }
        if self.allow_flying {
            byte |= 1 << 2;
        }
        if self.instant_break {
            byte |= 1 << 3;
        }
        buf.write_u8(byte);
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Position { x, y, z }
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_z(&self) -> f64 {
        self.z
    }

    pub fn set_x(&mut self, x: f64) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: f64) {
        self.y = y;
    }

    pub fn set_z(&mut self, z: f64) {
        self.z = z;
    }

    /// A chunk is 16 wide to this function, this is hardcoded
    pub fn get_chunk_x(&self) -> i32 {
        (self.x as i32) >> 4
    }

    /// A chunk is 16 long to this function, this is hardcoded
    pub fn get_chunk_z(&self) -> i32 {
        (self.z as i32) >> 4
    }

    pub fn get_chunk_coords(&self) -> (i32, i32) {
        (self.get_chunk_x(), self.get_chunk_z())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct LookAngles {
    yaw: f32,
    pitch: f32,
}

impl LookAngles {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        LookAngles { yaw, pitch }
    }

    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }

    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }
}
