use std::fmt::Write;
use std::fs::read_dir;
use std::path::PathBuf;
use std::{env, fs};

use convert_case::{Case, Casing};
use linked_hash_map::LinkedHashMap;

use crate::data::BlockData;
use crate::properties::{display_enum_properties, PropertyType};
use crate::raw::{collect_properties, RawBlockData};

mod data;
mod properties;
mod raw;

fn main() {
    if let Some(arg) = env::args().nth(1) {
        if arg == "props" {
            print_properties();
        } else {
            println!("Unrecognized option!");
        }
        return;
    }
    generate_code();
}

fn find_data_files() -> Vec<(i32, String)> {
    let files = read_dir(String::from(env!("CARGO_MANIFEST_DIR")) + "/raw_data").unwrap();
    let mut output = Vec::new();
    for file in files {
        let file = file.unwrap();
        let name = file.file_name().into_string().unwrap();
        // patterns should be "blocks-***.json" with "***" being the data version
        let (check, trimmed_front) = name.split_at(7);
        if check != "blocks-" {
            continue;
        }
        let trimmed = trimmed_front.split_at(trimmed_front.len() - 5).0;
        output.push((trimmed.parse::<i32>().unwrap(), name));
    }
    output.sort_by(|x1, x2| x2.0.cmp(&x1.0));
    output
}

fn get_data(filename: &str) -> LinkedHashMap<String, RawBlockData> {
    let mut work_dir = PathBuf::from(String::from(env!("CARGO_MANIFEST_DIR")) + "/raw_data");
    work_dir.push(filename);
    serde_json::from_str(&fs::read_to_string(work_dir).unwrap()).unwrap()
}

fn generate_code() {
    let files = find_data_files();
    let blocks = get_data(&files.get(0).unwrap().1);

    // base structure (highest data version)
    let base_parsed_data: Vec<(String, BlockData)> =
        blocks.into_iter().map(|x| (x.0, x.1.into())).collect();
    let mut output = String::new();
    let mut structs = String::new();
    writeln!(output, "#![allow(dead_code)]").unwrap();
    writeln!(output, "use std::str::FromStr;\nuse ahash::AHashMap;").unwrap();
    writeln!(output, "#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]").unwrap();
    writeln!(output, "pub enum Blocks {{").unwrap();
    for entry in &base_parsed_data {
        let pascal_name = entry.0.split_once(":").unwrap().1.to_case(Case::Pascal);
        let state_name = pascal_name.clone() + "State";
        if entry.1.properties.is_some() {
            writeln!(output, "    {}({}),", pascal_name, state_name).unwrap();
            entry.1.write_struct_def(&mut structs, &state_name).unwrap();
        } else {
            writeln!(output, "    {},", pascal_name).unwrap();
        }
    }
    writeln!(output, "}}").unwrap();

    let mut enum_appendix: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
    writeln!(output, "impl Blocks {{").unwrap();
    for (version, file_name) in files.iter().skip(1) {
        let raw_blocks = get_data(file_name);
        let raw_len = raw_blocks.len();
        // data check
        let mut count = 0;
        let mut valid_blocks = LinkedHashMap::new();
        for (name, entry) in raw_blocks {
            let parsed = BlockData::safe_from(entry);
            match parsed {
                None => {
                    count += 1;
                    println!("Could not parse {:?}", name);
                }
                Some(data) => match base_parsed_data.iter().find(|x| x.0.eq(&name)) {
                    Some(matched) => {
                        let mut found = true;
                        if let Some(block_state) = &data.properties {
                            if let Some(base_props) = matched.1.properties.as_ref() {
                                for prop in &block_state.properties {
                                    if !base_props.properties.contains(prop) {
                                        found = false;
                                        break;
                                    }
                                    if let PropertyType::Enum((target, real)) = &prop.property_type
                                    {
                                        if real
                                            .as_ref()
                                            .unwrap()
                                            .fields
                                            .iter()
                                            .enumerate()
                                            .any(|(i, x)| target.fields.get(i) != Some(x))
                                        {
                                            if let Some(list) = enum_appendix.get_mut(&name) {
                                                list.push(prop.name.clone());
                                            } else {
                                                enum_appendix
                                                    .insert(name.clone(), vec![prop.name.clone()]);
                                            }
                                        }
                                    }
                                }
                            } else {
                                found = false;
                            }
                        }
                        if !found {
                            count += 1;
                            println!(
                                "Could not find matched properties for {} due to {:?} vs {:?}",
                                name, matched.1.properties, data.properties
                            );
                        } else {
                            valid_blocks.insert(name, data);
                        }
                    }
                    None => {
                        count += 1;
                        println!("Could not match {}", name);
                    }
                },
            }
        }
        println!("{} were not matched for version {}", count, version);
        println!(
            "{}/{} successfully merged from version {}",
            valid_blocks.len(),
            raw_len,
            version
        );

        print_block_to_id(
            &mut output,
            &base_parsed_data,
            &valid_blocks,
            &enum_appendix,
            *version,
        );
    }
    println!("Found {} enum appendix entries", enum_appendix.len());
    print_base_blocks_to_id(&mut output, &base_parsed_data, files.get(0).unwrap().0);
    writeln!(output, "}}").unwrap();

    print_from_str(&mut output, &base_parsed_data);

    write!(output, "{}", structs).unwrap();
    display_enum_properties(&mut output);

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("blocks.rs");
    fs::write(&path, output).unwrap();
}

fn print_block_to_id<W: Write>(
    output: &mut W,
    base_blocks: &[(String, BlockData)],
    block_list: &LinkedHashMap<String, BlockData>,
    enum_appendix: &LinkedHashMap<String, Vec<String>>,
    version: i32,
) {
    writeln!(
        output,
        "    pub fn get_global_id_{}(&self) -> Option<i32> {{",
        version
    )
    .unwrap();
    writeln!(output, "        match self {{").unwrap();
    for (name, data) in base_blocks {
        let matched_block = block_list.get(name);
        let pascal_name = name.split_once(":").unwrap().1.to_case(Case::Pascal);
        match matched_block {
            None => {
                if data.properties.is_none() {
                    writeln!(output, "            Blocks::{} => None,", pascal_name).unwrap();
                } else {
                    writeln!(output, "            Blocks::{}(_) => None,", pascal_name).unwrap();
                }
            }
            Some(block_data) => match &data.properties {
                None => writeln!(
                    output,
                    "            Blocks::{} => Some({}),",
                    pascal_name, block_data.base_id
                )
                .unwrap(),
                Some(_props) => {
                    if let Some(properties) = &block_data.properties {
                        writeln!(output, "            Blocks::{}(state) => {{", pascal_name)
                            .unwrap();
                        let mut result = String::new();
                        write!(result, "                Some({}", block_data.base_id).unwrap();
                        let mut counter = 0;
                        let mut factor = 1;
                        let mut prev_count = 0;
                        for (i, property) in properties.properties.iter().rev().enumerate() {
                            if i > 0 {
                                factor *= prev_count;
                                write!(result, " + {} * ", factor).unwrap();
                            } else {
                                write!(result, " + ").unwrap();
                            }
                            prev_count = property.property_type.entry_count();
                            match &property.property_type {
                                PropertyType::Bool => {
                                    write!(result, "(!state.{}() as i32)", property.name).unwrap()
                                }
                                PropertyType::Int(_) => {
                                    write!(result, "state.{}", property.name).unwrap()
                                }
                                PropertyType::Enum((target, real)) => {
                                    let real = real.clone().unwrap();
                                    let mut changed_positions = false;
                                    if let Some(properties) = enum_appendix.get(name) {
                                        if properties.contains(&property.name) {
                                            writeln!(
                                                output,
                                                "                let value{} = match state.{} {{",
                                                counter, property.name
                                            )
                                            .unwrap();
                                            for (index, real_prop) in real.fields.iter().enumerate()
                                            {
                                                writeln!(
                                                    output,
                                                    "                    {}::{} => {},",
                                                    target.get_name(),
                                                    real_prop.to_case(Case::Pascal),
                                                    index
                                                )
                                                .unwrap();
                                            }
                                        }
                                        changed_positions = true;
                                    }
                                    if target.fields.len() > real.fields.len() {
                                        if changed_positions {
                                            writeln!(
                                                output,
                                                "                    _ => return None,"
                                            )
                                            .unwrap();
                                        } else {
                                            writeln!(
                                                output,
                                                "                if state.{} > {}::{} {{",
                                                property.name,
                                                target.get_name(),
                                                real.fields.last().unwrap().to_case(Case::Pascal)
                                            )
                                            .unwrap();
                                            writeln!(output, "                    return None;")
                                                .unwrap();
                                            writeln!(output, "                }}").unwrap();
                                        }
                                    }
                                    if changed_positions {
                                        writeln!(output, "                }};").unwrap();
                                        write!(result, "value{}", counter).unwrap();
                                        counter += 1;
                                    } else {
                                        write!(result, "(state.{} as i32)", property.name).unwrap();
                                    }
                                }
                            }
                        }
                        writeln!(output, "{})\n            }}", result).unwrap();
                    } else {
                        writeln!(
                            output,
                            "            Blocks::{}(_) => Some({}),",
                            pascal_name, block_data.base_id
                        )
                        .unwrap()
                    }
                }
            },
        }
    }
    writeln!(output, "        }}").unwrap();
    writeln!(output, "    }}").unwrap();
}

fn print_base_blocks_to_id<W: Write>(
    output: &mut W,
    block_list: &[(String, BlockData)],
    version: i32,
) {
    writeln!(
        output,
        "    pub fn get_global_id_{}(&self) -> i32 {{",
        version
    )
    .unwrap();
    writeln!(output, "        match self {{").unwrap();
    for (name, data) in block_list {
        let name = name.split_once(":").unwrap().1.to_case(Case::Pascal);
        match &data.properties {
            None => writeln!(
                output,
                "            Blocks::{} => {},",
                name, data.base_id
            )
            .unwrap(),
            Some(props) => {
                writeln!(output, "            Blocks::{}(state) => {{", name).unwrap();
                write!(output, "                {}", data.base_id).unwrap();
                let mut factor = 1;
                let mut prev_count = 0;
                for (i, property) in props.properties.iter().rev().enumerate() {
                    if i > 0 {
                        factor *= prev_count;
                        write!(output, " + {} * ", factor).unwrap();
                    } else {
                        write!(output, " + ").unwrap();
                    }
                    prev_count = property.property_type.entry_count();
                    match &property.property_type {
                        PropertyType::Bool => {
                            write!(output, "(!state.{}() as i32)", property.name).unwrap()
                        }
                        PropertyType::Int(_) => write!(output, "state.{}", property.name).unwrap(),
                        PropertyType::Enum(_) => {
                            write!(output, "(state.{} as i32)", property.name).unwrap()
                        }
                    }
                }
                writeln!(output, "\n            }}").unwrap();
            }
        }
    }
    writeln!(output, "        }}").unwrap();
    writeln!(output, "    }}").unwrap();
}

fn print_from_str<W: Write>(output: &mut W, block_list: &[(String, BlockData)]) {
    writeln!(output, "#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]").unwrap();
    writeln!(output, "pub enum ParseBlockError {{").unwrap();
    writeln!(output, "    UnknownBlock,\n    UnknownProperty,\n    InvalidProperty,\n    InvalidToken,").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output, "impl ::std::error::Error for ParseBlockError {{}}").unwrap();
    writeln!(output, "impl ::std::fmt::Display for ParseBlockError {{
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {{
        write!(f, \"{{:?}}\", self)
    }}
}}").unwrap();
    writeln!(output, "impl From<std::str::ParseBoolError> for ParseBlockError {{
    fn from(_: std::str::ParseBoolError) -> Self {{
        ParseBlockError::InvalidProperty
    }}
}}").unwrap();
    writeln!(output, "impl From<std::num::ParseIntError> for ParseBlockError {{
    fn from(_: std::num::ParseIntError) -> Self {{
        ParseBlockError::InvalidProperty
    }}
}}").unwrap();
    writeln!(output, "impl FromStr for Blocks {{").unwrap();
    writeln!(output, "    type Err = ParseBlockError;").unwrap();
    writeln!(output, "    fn from_str(s: &str) -> Result<Self, Self::Err> {{").unwrap();
    writeln!(output, "        let (_domain, stripped) = if s.contains(':') {{ s.split_once(':').unwrap() }} else {{ (\"\", s) }};").unwrap();
    writeln!(output, "        let (name, stripped) = if let Some(i) = stripped.rfind(']') {{
            let _ = stripped.find('[').ok_or(ParseBlockError::InvalidToken)?;
            stripped.split_at(i).0.split_once('[').unwrap()
        }} else {{
            (stripped, \"\")
        }};").unwrap();
    writeln!(output, "        let props: AHashMap<&str, &str> = stripped.split(',')
            .map(|x| x.split_once('=')).flatten().collect();").unwrap();
    writeln!(output, "        Ok(match name {{").unwrap();
    for (name, data) in block_list {
        let clean_name = name.split_once(":").unwrap().1;
        if let Some(properties) = &data.properties {
            writeln!(output, "            \"{}\" => {{", clean_name).unwrap();
            writeln!(output, "                let mut block_state = {}::default();", clean_name.to_case(Case::Pascal) + "State").unwrap();
            for property in &properties.properties {
                writeln!(output, "                if let Some(prop) = props.get(\"{}\") {{", property.name).unwrap();
                match &property.property_type {
                    PropertyType::Bool => writeln!(output, "                    block_state.with_{}(bool::from_str(prop)?);", property.name).unwrap(),
                    PropertyType::Int(_) => writeln!(output, "                    block_state.with_{}(i32::from_str(prop)?);", property.name).unwrap(),
                    PropertyType::Enum((enum_prop, _)) => writeln!(output, "                    block_state.with_{}({}::from_str(prop)?);", property.name, enum_prop.get_name()).unwrap(),
                }
                writeln!(output, "                }}").unwrap();
            }
            writeln!(output, "                Blocks::{}(block_state)", clean_name.to_case(Case::Pascal)).unwrap();
            writeln!(output, "            }}").unwrap();
        } else {
            writeln!(output, "            \"{}\" => Blocks::{},", clean_name, clean_name.to_case(Case::Pascal)).unwrap();
        }
    }
    write!(output, "            _ => return Err(ParseBlockError::UnknownBlock),").unwrap();
    writeln!(output, "        }})\n    }}").unwrap();
    writeln!(output, "}}").unwrap();
}

fn print_properties() {
    let files = find_data_files();
    let blocks = get_data(&files.get(0).unwrap().1);
    let sorted_props = collect_properties(&blocks);
    println!("Found {} property values:", sorted_props.len());
    for prop in &sorted_props {
        println!("{:?}", prop);
    }

    println!("\nDuplicates found: ");
    for i in 0..sorted_props.len() {
        if sorted_props[..].iter().any(|x| {
            x != &sorted_props[i]
                && !x
                    .values
                    .iter()
                    .any(|z| z == "true" || z.parse::<i8>().is_ok())
                && !sorted_props[i].values.iter().any(|z| z == "true")
                && sorted_props[i].names.iter().any(|y| x.names.contains(y))
        }) {
            let entry = &sorted_props[i];
            println!("{:?}", entry);
        }
    }
}
