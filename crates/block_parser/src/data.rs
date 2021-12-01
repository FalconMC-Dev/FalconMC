use std::fmt::Write;

use convert_case::{Case, Casing};

use crate::properties::PropertyType;
use crate::raw::RawBlockData;

#[derive(Debug)]
pub struct BlockData {
    pub base_id: i32,
    pub properties: Option<BlockState>,
}

impl BlockData {
    pub fn write_struct_def<W: Write>(&self, output: &mut W, name: &String) -> std::fmt::Result {
        let block_state = self.properties.as_ref().unwrap();
        // struct definition
        write!(output, "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub struct {} {{\n", name)?;
        for property in &block_state.properties {
            write!(output, "    {}: {},\n", property.name, property.property_type)?;
        }
        write!(output, "}}\n")?;
        // getters and setters
        write!(output, "impl {} {{\n", name)?;
        for property in &block_state.properties {
            if let PropertyType::Int(ref range) = property.property_type {
                write!(output, "    /// This is a value between {} and {} (both ends inclusive).\\Developers should be careful to respect these bounds as no checking is done at runtime!!!\n", range.start, range.end - 1)?;
            }
            write!(output, "    pub fn with_{}(&mut self, {}: {}) -> &mut Self {{\n", property.name, property.name, property.property_type)?;
            if let PropertyType::Int(ref range) = property.property_type {
                if range.start != 0 {
                    write!(output, "        self.{} = {} - {};\n", property.name, property.name, range.start)?;
                } else {
                    write!(output, "        self.{} = {};\n", property.name, property.name)?;
                }
            } else {
                write!(output, "        self.{} = {};\n", property.name, property.name)?;
            }
            write!(output, "        self\n")?;
            write!(output, "    }}\n")?;
            write!(output, "    pub fn {}(&self) -> {} {{\n", property.name, property.property_type)?;
            if let PropertyType::Int(ref range) = property.property_type {
                if range.start != 0 {
                    write!(output, "        self.{} + {}\n", property.name, range.start)?;
                } else {
                    write!(output, "        self.{}\n", property.name)?;
                }
            } else {
                write!(output, "        self.{}\n", property.name)?;
            }
            write!(output, "    }}\n")?;
        }
        write!(output, "}}\n")?;
        // default implementation
        write!(output, "impl Default for {} {{\n    fn default() -> Self {{\n", name)?;
        write!(output, "        {} {{\n", name)?;
        for entry in &block_state.default {
            let property = block_state.properties.iter().find(|x| x.name == entry.name).unwrap();
            write!(output, "            {}: ", property.name)?;
            match &property.property_type {
                PropertyType::Bool => write!(output, "{},\n", entry.value)?,
                PropertyType::Int(range) => {
                    if range.start != 0 {
                        let value = entry.value.parse::<u8>().unwrap() - range.start;
                        write!(output, "{},\n", value)?
                    } else {
                        write!(output, "{},\n", entry.value)?
                    }
                },
                PropertyType::Enum((target, _real)) => write!(output, "{}::{},\n", target.get_name(), entry.value.to_case(Case::Pascal))?,
            }
        }
        write!(output, "        }}\n    }}\n}}\n")
    }
}

impl From<RawBlockData> for BlockData {
    fn from(raw: RawBlockData) -> Self {
        BlockData {
            base_id: raw.states.get(0).unwrap().id,
            properties: raw.try_into().map_or(None, |x| Some(x)),
        }
    }
}

impl BlockData {
    pub fn safe_from(raw: RawBlockData) -> Option<Self> {
        let base_id = raw.states.get(0).unwrap().id;
        let properties = match BlockState::safe_from(raw) {
            Ok(props) => props,
            Err(_) => return None,
        };
        Some(BlockData {
            base_id,
            properties
        })
    }
}

#[derive(Debug, Eq)]
pub struct BlockState {
    pub properties: Vec<BlockProperty>,
    default: Vec<RawPropertyValue>,
}

impl TryFrom<RawBlockData> for BlockState {
    type Error = ();
    fn try_from(mut raw: RawBlockData) -> Result<Self, Self::Error> {
        let properties: Vec<BlockProperty> = raw.properties.ok_or(())?.into_iter().map(|entry| entry.into()).collect();
        // properties.sort_by(|x1, x2| x1.raw.cmp(&x2.raw)); // should be unnecessary
        let raw_default = raw.states.drain(..).find(|x| x.default.is_some()).unwrap();
        let default: Vec<RawPropertyValue> = raw_default.properties.unwrap().into_iter().map(|x| RawPropertyValue::new(avoid_type(x.0.to_case(Case::Snake)), x.1)).collect();
        Ok(BlockState {
            properties,
            default,
        })
    }
}

impl PartialEq for BlockState {
    fn eq(&self, other: &Self) -> bool {
        self.properties.eq(&other.properties)
    }
}

impl BlockState {
    pub fn safe_from(mut raw: RawBlockData) -> Result<Option<Self>, ()> {
        if raw.properties.is_none() {
            return Ok(None);
        }
        let mut properties: Vec<BlockProperty> = Vec::new();
        for entry in raw.properties.unwrap().into_iter() {
            properties.push(BlockProperty::try_from(entry)?);
        }
        let raw_default = raw.states.drain(..).find(|x| x.default.is_some()).unwrap();
        let default: Vec<RawPropertyValue> = raw_default.properties.unwrap().into_iter().map(|x| RawPropertyValue::new(avoid_type(x.0.to_case(Case::Snake)), x.1)).collect();
        Ok(Some(BlockState {
            properties,
            default,
        }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BlockProperty {
    raw: String,
    pub name: String,
    pub property_type: PropertyType,
}

impl From<(String, Vec<String>)> for BlockProperty {
    fn from(raw: (String, Vec<String>)) -> Self {
        BlockProperty {
            raw: raw.0.clone(),
            name: avoid_type(raw.0.to_case(Case::Snake)),
            property_type: PropertyType::from_raw(raw.0, raw.1)
        }
    }
}

impl BlockProperty {
    pub fn try_from(raw: (String, Vec<String>)) -> Result<Self, ()> {
        let raw_name = raw.0.clone();
        let name = avoid_type(raw.0.to_case(Case::Snake));
        let property_type = PropertyType::find(raw.0, raw.1).ok_or(())?;
        Ok(BlockProperty {
            raw: raw_name,
            name,
            property_type,
        })
    }
}

fn avoid_type(mut input: String) -> String {
    if input == "type" {
        input.push('d');
    }
    input
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawPropertyValue {
    name: String,
    value: String,
}

impl RawPropertyValue {
    pub fn new(name: String, value: String) -> Self {
        RawPropertyValue {
            name,
            value,
        }
    }
}