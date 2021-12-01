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
#[cfg(test)]
mod tests;

fn main() {
    if let Some(arg) = env::args().skip(1).next() {
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
    write!(output, "#![allow(dead_code)]\n").unwrap();
    write!(output, "pub enum Blocks {{\n").unwrap();
    for entry in &base_parsed_data {
        let pascal_name = entry.0.split_once(":").unwrap().1.to_case(Case::Pascal);
        let state_name = String::from(pascal_name.clone() + "State");
        if entry.1.properties.is_some() {
            write!(output, "    {}({}),\n", pascal_name, state_name).unwrap();
            entry.1.write_struct_def(&mut structs, &state_name).unwrap();
        } else {
            write!(output, "    {},\n", pascal_name).unwrap();
        }
    }
    write!(output, "}}\n").unwrap();

    let mut enum_appendix: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
    write!(output, "impl Blocks {{\n").unwrap();
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
                                    if !base_props.properties.contains(&prop) {
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
    write!(output, "}}\n").unwrap();

    write!(output, "{}", structs).unwrap();
    display_enum_properties(&mut output);

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("blocks.rs");
    fs::write(&path, output).unwrap();
}

fn print_block_to_id<W: Write>(
    output: &mut W,
    base_blocks: &Vec<(String, BlockData)>,
    block_list: &LinkedHashMap<String, BlockData>,
    enum_appendix: &LinkedHashMap<String, Vec<String>>,
    version: i32,
) {
    write!(
        output,
        "    pub fn get_global_id_{}(&self) -> Option<i32> {{\n",
        version
    )
    .unwrap();
    write!(output, "        match self {{\n").unwrap();
    for (name, data) in base_blocks {
        let matched_block = block_list.get(name);
        let pascal_name = name.split_once(":").unwrap().1.to_case(Case::Pascal);
        match matched_block {
            None => {
                if data.properties.is_none() {
                    write!(output, "            Blocks::{} => None,\n", pascal_name).unwrap();
                } else {
                    write!(output, "            Blocks::{}(_) => None,\n", pascal_name).unwrap();
                }
            }
            Some(block_data) => match &data.properties {
                None => write!(
                    output,
                    "            Blocks::{} => Some({}),\n",
                    pascal_name, block_data.base_id
                )
                .unwrap(),
                Some(_props) => {
                    if let Some(properties) = &block_data.properties {
                        write!(output, "            Blocks::{}(state) => {{\n", pascal_name)
                            .unwrap();
                        let mut result = String::new();
                        write!(result, "                Some({}", block_data.base_id).unwrap();
                        let mut counter = 0;
                        for (i, property) in properties.properties.iter().rev().enumerate() {
                            if i > 0 {
                                write!(result, " + {} * ", i + 1).unwrap();
                            } else {
                                write!(result, " + ").unwrap();
                            }
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
                                            write!(
                                                output,
                                                "                let value{} = match state.{} {{\n",
                                                counter, property.name
                                            )
                                            .unwrap();
                                            for (index, real_prop) in real.fields.iter().enumerate()
                                            {
                                                write!(
                                                    output,
                                                    "                    {}::{} => {},\n",
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
                                            write!(
                                                output,
                                                "                    _ => return None,\n"
                                            )
                                            .unwrap();
                                        } else {
                                            write!(
                                                output,
                                                "                if state.{} > {}::{} {{\n",
                                                property.name,
                                                target.get_name(),
                                                real.fields.last().unwrap().to_case(Case::Pascal)
                                            )
                                            .unwrap();
                                            write!(output, "                    return None;\n")
                                                .unwrap();
                                            write!(output, "                }}\n").unwrap();
                                        }
                                    }
                                    if changed_positions {
                                        write!(output, "                }};\n").unwrap();
                                        write!(result, "value{}", counter).unwrap();
                                        counter += 1;
                                    } else {
                                        write!(result, "(state.{} as i32)", property.name).unwrap();
                                    }
                                }
                            }
                        }
                        write!(output, "{})\n            }}\n", result).unwrap();
                    } else {
                        write!(
                            output,
                            "            Blocks::{}(_) => Some({}),\n",
                            pascal_name, block_data.base_id
                        )
                        .unwrap()
                    }
                }
            },
        }
    }
    write!(output, "        }}\n").unwrap();
    write!(output, "    }}\n").unwrap();
}

fn print_base_blocks_to_id<W: Write>(
    output: &mut W,
    block_list: &Vec<(String, BlockData)>,
    version: i32,
) {
    write!(
        output,
        "    pub fn get_global_id_{}(&self) -> i32 {{\n",
        version
    )
    .unwrap();
    write!(output, "        match self {{\n").unwrap();
    for (name, data) in block_list {
        let name = name.split_once(":").unwrap().1.to_case(Case::Pascal);
        match &data.properties {
            None => write!(
                output,
                "            Blocks::{} => {},\n",
                name, data.base_id
            )
            .unwrap(),
            Some(props) => {
                write!(output, "            Blocks::{}(state) => {{\n", name).unwrap();
                write!(output, "                {}", data.base_id).unwrap();
                for (i, property) in props.properties.iter().rev().enumerate() {
                    if i > 0 {
                        write!(output, " + {} * ", i + 1).unwrap();
                    } else {
                        write!(output, " + ").unwrap();
                    }
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
                write!(output, "\n            }}\n").unwrap();
            }
        }
    }
    write!(output, "        }}\n").unwrap();
    write!(output, "    }}\n").unwrap();
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
