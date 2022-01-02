use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use crate::network::buffer::PacketBufferWrite;

use crate::network::connection::ConnectionTask;
use crate::network::packet::PacketEncode;

pub trait MinecraftPlayer {
    /// identity methods
    fn get_username(&self) -> &str;

    fn get_uuid(&self) -> Uuid;

    /// game methods
    fn get_entity_id(&self) -> i32;

    fn get_game_mode(&self) -> GameMode;

    fn get_dimension(&self) -> i32;

    fn get_ability_flags(&self) -> PlayerAbilityFlags;

    fn get_position(&self) -> &Position;

    fn get_position_mut(&mut self) -> &mut Position;

    fn get_look_angles(&self) -> &LookAngles;

    fn get_look_angles_mut(&mut self) -> &mut LookAngles;

    /// connection methods
    fn get_protocol_version(&self) -> i32;

    fn disconnect(&mut self, reason: String);

    fn get_client_connection(&mut self) -> &mut UnboundedSender<Box<ConnectionTask>>;
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
            instant_break
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

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Position {
            x,
            y,
            z
        }
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
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LookAngles {
    yaw: f32,
    pitch: f32,
}

impl LookAngles {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        LookAngles {
            yaw,
            pitch
        }
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

