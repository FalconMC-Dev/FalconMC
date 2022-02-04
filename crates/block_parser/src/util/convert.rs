use convert_case::{Case, Casing};
use crate::util::raw::RawBlockData;
use crate::util::avoid_type;
use crate::util::data::{BlockData, BlockProperty, BlockState};
use crate::util::properties::{EnumProperty, EnumPropertyBase, PropertyType};

pub fn into_block_data(raw_data: &RawBlockData, property_base: &mut EnumPropertyBase, default_version: bool) -> BlockData {
    if raw_data.properties().is_none() {
        return BlockData::new(raw_data.base_id(), vec![], BlockState::new(vec![]));
    }

    let properties = determine_properties(raw_data, property_base, default_version);
    let raw_base_state = raw_data.base_state();
    let mut default_state = Vec::new();
    for (name, value) in raw_base_state.properties().unwrap() {
        let found_property = properties.iter().find(|x| x.name() == &avoid_type(name.clone())).unwrap();
        default_state.push((found_property.clone(), value.clone()));
    }
    BlockData::new(raw_data.base_id(), properties, BlockState::new(default_state))
}

fn determine_properties(raw_data: &RawBlockData, property_base: &mut EnumPropertyBase, default_version: bool) -> Vec<BlockProperty> {
    let mut properties = Vec::new();

    let raw_properties = raw_data.properties();
    if let Some(raw_properties) = raw_properties {
        for (raw_name, raw_values) in raw_properties.iter() {
            let property_name = avoid_type(raw_name.to_case(Case::Snake));
            let value1 = raw_values.first().unwrap();
            // case 1: boolean property
            if value1 == "true" || value1 == "false" {
                properties.push(BlockProperty::new(property_name, PropertyType::Bool));
            // case 2: integer range
            } else if let Ok(start) = value1.parse::<u8>() {
                properties.push(BlockProperty::new(property_name, PropertyType::Int(start..(raw_values.last().map(|x| x.parse::<u8>().unwrap()).unwrap() + 1))));
            // case 3: enum property
            } else {
                match property_base.find_property(&property_name, raw_values) {
                    None => {
                        if default_version {
                            let property = property_base.add(EnumProperty::new(raw_name.to_case(Case::Pascal), raw_values.clone()));
                            properties.push(BlockProperty::new(property_name, PropertyType::Enum(property)));
                        } else {
                            match property_base.find_fields(raw_values) {
                                Some(second_chance) => {
                                    let enum_property = EnumProperty::with_order(String::from(second_chance.name()), raw_values.clone(), second_chance.fields_cloned());
                                    let block_property = BlockProperty::new(property_name, PropertyType::Enum(enum_property));
                                    properties.push(block_property);
                                },
                                None => {
                                    println!("Could not find property {}!", property_name)
                                }
                            }
                        }
                    }
                    Some(base_property) => {
                        if default_version {
                            properties.push(BlockProperty::new(property_name, PropertyType::Enum(base_property.clone())));
                        } else {
                            for (i, entry) in raw_values.iter().enumerate() {
                                if base_property.get(i) != Some(entry) {
                                    println!("Fields cannot be merged for {}!", property_name);
                                    continue;
                                }
                            }
                            if raw_values.len() < base_property.entry_count() {
                                properties.push(BlockProperty::new(property_name, PropertyType::Enum(EnumProperty::with_order(String::from(base_property.name()), raw_values.clone(), base_property.fields_cloned()))));
                            } else {
                                properties.push(BlockProperty::new(property_name, PropertyType::Enum(base_property.clone())));
                            }
                        }
                    }
                }
            }
        }
    }
    properties
}