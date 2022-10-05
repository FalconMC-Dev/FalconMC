use std::borrow::Cow;
use std::str::FromStr;

use ahash::AHashMap;
use fastnbt::borrow::{ByteArray, IntArray};
use serde::Deserialize;

use crate::error::FalconCoreError;
use crate::world::blocks::Blocks;

pub const REQUIRED_DATA_VERSION: i32 = 2730;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SchematicVersionedRaw<'a> {
    // version
    version: i32,
    data_version: Option<i32>,
    // mutual data
    width: i16,
    height: i16,
    length: i16,
    #[serde(borrow)]
    offset: Option<IntArray<'a>>,
    palette: AHashMap<Cow<'a, str>, i32>,
    #[serde(borrow)]
    block_data: Option<ByteArray<'a>>,
}

pub struct SchematicData<'a> {
    pub width: u16,
    pub height: u16,
    pub length: u16,
    pub offset: [i32; 3],
    pub palette: AHashMap<i32, Blocks>,
    pub block_data: ByteArray<'a>,
}

impl<'a> SchematicData<'a> {
    pub fn new(width: u16, height: u16, length: u16, offset: [i32; 3], palette: AHashMap<i32, Blocks>, block_data: ByteArray<'a>) -> Self {
        SchematicData {
            width,
            height,
            length,
            offset,
            palette,
            block_data,
        }
    }
}

impl<'a> TryFrom<SchematicVersionedRaw<'a>> for SchematicData<'a> {
    type Error = FalconCoreError;

    fn try_from(value: SchematicVersionedRaw<'a>) -> std::result::Result<Self, Self::Error> {
        if value.version != 2 {
            return Err(FalconCoreError::InvalidSchematic(value.version));
        }
        match value.data_version {
            Some(content) => {
                if content != REQUIRED_DATA_VERSION {
                    return Err(FalconCoreError::WrongDataVersion(REQUIRED_DATA_VERSION, content));
                }
            },
            None => return Err(FalconCoreError::MissingData),
        }
        let block_data = {
            match value.block_data {
                Some(data) => data,
                None => return Err(FalconCoreError::MissingData),
            }
        };

        let mut effective_offset = [0; 3];
        if let Some(offset) = value.offset {
            if offset.iter().count() != 3 {
                return Err(FalconCoreError::InvalidData(String::from("Expected 3 offset coords")));
            }
            offset.iter().enumerate().for_each(|(i, x)| effective_offset[i] = x);
        }

        let mut effective_palette = AHashMap::new();
        for (state, index) in value.palette {
            effective_palette.insert(index, Blocks::from_str(state.as_ref())?);
        }

        Ok(SchematicData::new(value.width as u16, value.height as u16, value.length as u16, effective_offset, effective_palette, block_data))
    }
}
