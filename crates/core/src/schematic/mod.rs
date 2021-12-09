use std::borrow::Cow;
use std::io::Cursor;
use std::str::FromStr;

use ahash::AHashMap;
use error_chain::{bail, ensure};
use fastnbt::borrow::IntArray;
use fastnbt::ByteArray;
use serde::Deserialize;

use crate::errors::*;
use crate::network::buffer::{PacketBufferRead, as_u8_slice};
use crate::world::blocks::Blocks;

pub const REQUIRED_DATA_VERSION: i32 = 2730;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SchematicVersionedRaw<'a> {
    // version
    version: i32,
    data_version: Option<i32>,
    // mutual data
    width: u16,
    height: u16,
    length: u16,
    #[serde(borrow)]
    offset: Option<IntArray<'a>>,
    palette: AHashMap<Cow<'a, str>, i32>,
    block_data: Option<ByteArray>,
}

pub struct SchematicData {
    width: u16,
    height: u16,
    length: u16,
    offset: [i32; 3],
    palette: AHashMap<i32, Blocks>,
    block_data: Vec<i32>,
}

impl SchematicData {
    pub fn new(width: u16, height: u16, length: u16, offset: [i32; 3], palette: AHashMap<i32, Blocks>, block_data: Vec<i32>) -> Self {
        SchematicData {
            width,
            height,
            length,
            offset,
            palette,
            block_data
        }
    }
}

impl<'a> TryFrom<SchematicVersionedRaw<'a>> for SchematicData {
    type Error = Error;

    fn try_from(value: SchematicVersionedRaw<'a>) -> std::result::Result<Self, Self::Error> {
        ensure!(value.version < 2 && value.version > 0, ErrorKind::InvalidSchematicVersion(value.version));
        match value.data_version {
            Some(content) => if content != REQUIRED_DATA_VERSION { bail!(ErrorKind::WrongDataVersion(content, REQUIRED_DATA_VERSION))},
            None => bail!(ErrorKind::MissingData),
        }
        ensure!(value.block_data.is_some(), ErrorKind::MissingData);
        let block_data = value.block_data.unwrap();

        let mut effective_offset = [0; 3];
        if let Some(offset) = value.offset {
            ensure!(offset.iter().count() == 3, ErrorKind::InvalidData);
            offset.iter().enumerate().for_each(|(i, x)| effective_offset[i] = x);
        }

        let mut effective_palette = AHashMap::new();
        for (state, index) in value.palette {
            effective_palette.insert(index, Blocks::from_str(state.as_ref()).chain_err(|| "Invalid BlockState")?);
        }

        let mut effective_block_data = Vec::with_capacity((value.width * value.height * value.length) as usize);
        let mut data_cursor = Cursor::new(as_u8_slice(block_data.as_slice()));
        let mut stop = false;
        while !stop {
            match data_cursor.read_var_i32() {
                Ok(int) => effective_block_data.push(int),
                Err(err) => {
                    match err.kind() {
                        ErrorKind::NoMoreBytes => stop = true,
                        _ => return Err(err),
                    }
                }
            }
        }

        Ok(SchematicData::new(value.width, value.height, value.length, effective_offset, effective_palette, effective_block_data))
    }
}
