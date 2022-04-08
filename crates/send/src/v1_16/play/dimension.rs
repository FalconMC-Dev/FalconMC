use falcon_core::world::dimension::Dimension;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

#[derive(Serialize)]
pub struct Codec {
    dimension: Vec<DimensionData>,
}

impl Codec {
    pub fn new(dimension: Vec<DimensionData>) -> Self {
        Codec {
            dimension,
        }
    }
}

pub struct DimensionData {
    dimension: Dimension,
}

impl DimensionData {
    pub fn new(dimension: Dimension) -> Self {
        DimensionData {
            dimension,
        }
    }
}

impl Serialize for DimensionData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut serializer = serializer.serialize_struct("dimension", 13)?;
        serializer.serialize_field("name", self.dimension.name())?;
        serializer.serialize_field("natural", &true)?;
        serializer.serialize_field("ambient_light", &0.5f32)?;
        serializer.serialize_field("has_ceiling", &false)?;
        serializer.serialize_field("has_skylight", &true)?;
        serializer.serialize_field("shrunk", &false)?;
        serializer.serialize_field("ultrawarm", &false)?;
        serializer.serialize_field("has_raids", &true)?;
        serializer.serialize_field("respawn_anchor_works", &true)?;
        serializer.serialize_field("bed_works", &true)?;
        serializer.serialize_field("piglin_safe", &true)?;
        serializer.serialize_field("logical_height", &256i32)?;
        serializer.serialize_field("infiniburn", "minecraft:infiniburn_overworld")?;
        serializer.end()
    }
}
