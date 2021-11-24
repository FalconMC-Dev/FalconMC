use std::collections::HashMap;
use std::{env, fs};
use std::path::PathBuf;
use convert_case::{Case, Casing};
use crate::raw::collect_properties;
use crate::{BlockData, RawBlockData};
use crate::properties::{display_enum_properties, PropertyType};

fn get_data() -> HashMap<String, RawBlockData> {
    let mut work_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    work_dir.push("blocks-1.17.1.json");
    serde_json::from_str(&fs::read_to_string(work_dir).unwrap()).unwrap()
}

#[test]
fn generate_code() {
    let mut blocks = get_data();
    let parsed_data: Vec<(String, BlockData)> = blocks.drain().map(|x| (x.0, x.1.into())).collect();
    let mut output = String::new();
    let entry = parsed_data.iter().filter(|x| x.1.properties.is_some()).next().unwrap();
    let name = &entry.0;
    entry.1.write_struct_def(&mut output, &name.split_once(":").unwrap().1.to_case(Case::Pascal)).unwrap();
    println!("{}", output);
    /*for data in parsed_data {
        println!("{:?}", data);
    }*/
}

#[test]
fn print_properties() {
    let blocks = get_data();
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