use crate::player::data::Position;

pub mod blocks;
pub mod block_util;
pub mod chunks;
pub mod dimension;
pub mod palette;

#[derive(Clone, Copy, Debug)]
pub struct BlockPosition {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        BlockPosition { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn set_z(&mut self, z: i32) {
        self.z = z;
    }
}

impl From<Position> for BlockPosition {
    fn from(pos: Position) -> Self {
        BlockPosition::new(
            pos.x().floor() as i32,
            pos.y().floor() as i32,
            pos.z().floor() as i32,
        )
    }
}
