use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Peaceful = 0,
    Easy,
    Normal,
    Hard,
}

impl From<Difficulty> for u8 {
    fn from(src: Difficulty) -> Self { src as u8 }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameMode {
    Survival = 0,
    Creative,
    Adventure,
    Spectator,
}

impl From<GameMode> for u8 {
    fn from(src: GameMode) -> Self { src as u8 }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct PlayerAbilityFlags {
    pub invulnerable: bool,
    pub is_flying: bool,
    pub is_flight: bool,
    pub instant_break: bool,
}

impl PlayerAbilityFlags {
    pub fn new(invulnerable: bool, is_flying: bool, is_flight: bool, instant_break: bool) -> Self {
        PlayerAbilityFlags {
            invulnerable,
            is_flying,
            is_flight,
            instant_break,
        }
    }
}

impl From<PlayerAbilityFlags> for u8 {
    fn from(flags: PlayerAbilityFlags) -> Self {
        flags.invulnerable as u8 | (flags.is_flight as u8) << 1 | (flags.is_flight as u8) << 2 | (flags.instant_break as u8) << 3
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self { Position { x, y, z } }

    /// A chunk is 16 wide to this function, this is hardcoded
    pub fn chunk_x(&self) -> i32 { (self.x as i32) >> 4 }

    /// A chunk is 16 long to this function, this is hardcoded
    pub fn chunk_z(&self) -> i32 { (self.z as i32) >> 4 }

    pub fn chunk(&self) -> (i32, i32) { (self.chunk_x(), self.chunk_z()) }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct LookAngles {
    pub yaw: f32,
    pub pitch: f32,
}

impl LookAngles {
    pub fn new(yaw: f32, pitch: f32) -> Self { LookAngles { yaw, pitch } }
}
