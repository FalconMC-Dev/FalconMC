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
                PropertyType::Enum(enum_property) => write!(output, "{}::{},\n", enum_property.get_name(), entry.value.to_case(Case::Pascal))?,
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

#[derive(Debug)]
pub struct BlockState {
    properties: Vec<BlockProperty>,
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

#[derive(Debug)]
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

fn avoid_type(mut input: String) -> String {
    if input == "type" {
        input.push('d');
    }
    input
}

#[derive(Debug)]
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