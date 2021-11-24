use std::fmt::Write;
use std::{env, fs};
use std::fs::read_dir;
use std::path::PathBuf;

use convert_case::{Case, Casing};
use linked_hash_map::LinkedHashMap;

use crate::data::BlockData;
use crate::properties::display_enum_properties;
use crate::raw::{collect_properties, RawBlockData};

mod data;
mod raw;
mod properties;
#[cfg(test)]
mod tests;

fn main() {
    if let Some(arg) = env::args().skip(1).next() {
        if arg == "props" {
            print_properties();
            return;
        }
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
    let parsed_data: Vec<(String, BlockData)> = blocks.into_iter().map(|x| (x.0, x.1.into())).collect();
    let mut output = String::new();
    let mut structs = String::new();
    write!(output, "#![allow(dead_code)]\n").unwrap();
    write!(output, "pub enum Blocks {{\n").unwrap();
    for ref entry in parsed_data {
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
    write!(output, "{}", structs).unwrap();
    display_enum_properties(&mut output);

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("blocks.rs");
    fs::write(&path, output).unwrap();
}

fn print_properties() {
    let files = find_data_files();
    let blocks = get_data(files.iter().max_by(|x1, x2| x1.1.cmp(&x2.1)).map(|x| &x.1).unwrap());
    let sorted_props = collect_properties(&blocks);
    println!("Found {} property values:", sorted_props.len());
    for prop in &sorted_props {
        println!("{:?}", prop);
    }

    println!("\nDuplicates found: ");
    for i in 0..sorted_props.len() {
        if sorted_props[..].iter().any(|x| {
            x != &sorted_props[i]
                && !x.values.iter().any(|z| z == "true" || z.parse::<i8>().is_ok())
                && !sorted_props[i].values.iter().any(|z| z == "true")
                && sorted_props[i].names.iter().any(|y| x.names.contains(y))
        }) {
            let entry = &sorted_props[i];
            println!("{:?}", entry);
        }
    }
}