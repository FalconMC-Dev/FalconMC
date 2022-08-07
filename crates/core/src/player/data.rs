use crate::network::buffer::PacketBufferWrite;
use crate::network::packet::PacketEncode;
use serde::{Deserialize, Serialize};

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

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
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
    pub fn chunk_x(&self) -> i32 {
        (self.x as i32) >> 4
    }

    /// A chunk is 16 long to this function, this is hardcoded
    pub fn chunk_z(&self) -> i32 {
        (self.z as i32) >> 4
    }

    pub fn chunk_coords(&self) -> (i32, i32) {
        (self.chunk_x(), self.chunk_z())
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

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }
}
