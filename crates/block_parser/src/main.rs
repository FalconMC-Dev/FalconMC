use ahash::AHashMap;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::{env, fs};

use functionality::{find_data_files, get_data, load_block_lists};
use linked_hash_map::LinkedHashMap;
use proc_macro2::TokenStream;
use quote::quote;
use util::generate_block_parse_error;
use util::raw::RawBlockData;

pub mod functionality;
pub mod util;

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

fn generate_code() {
    let (versions, base_set) = load_block_lists();
    let latest = versions.get(0).unwrap();

    let block_enum_definition = latest.generate_enum_definition();
    let type_enum_definitions = base_set.generate_enum_list();
    let struct_definitions = latest.generate_struct_definitions();
    let error_definition = generate_block_parse_error();
    let str_to_block_function = latest.generate_from_str_impl();
    let convert_to_id_functions: Vec<TokenStream> = versions
        .iter()
        .enumerate()
        .map(|(i, version)| {
            if i > 0 {
                version.generate_block_to_id_definition(Some(&latest))
            } else {
                version.generate_block_to_id_definition(None)
            }
        })
        .collect();

    let block_file_code = quote!(
        #![allow(clippy::derivable_impls)]
        #block_enum_definition

        impl Blocks {
            #(#convert_to_id_functions)*
        }

        #struct_definitions

        #error_definition
        #str_to_block_function

        #type_enum_definitions
    );

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("blocks-new.rs");
    fs::write(&path, rustfmt_wrapper::rustfmt(block_file_code).unwrap()).unwrap();
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

pub fn collect_properties(data: &LinkedHashMap<String, RawBlockData>) -> Vec<RawPropertyList> {
    let mut property_values = AHashMap::new();
    for (_, entry) in data.iter() {
        if let Some(map) = entry.properties() {
            for (name, values) in map.iter() {
                if !property_values.contains_key(values) {
                    let list = vec![name.clone()];
                    property_values.insert(values.clone(), list);
                } else {
                    let value = property_values.get_mut(values).unwrap();
                    if !value.contains(name) {
                        value.push(name.clone());
                    }
                }
            }
        }
    }
    let mut results: Vec<RawPropertyList> = property_values
        .drain()
        .map(|entry| RawPropertyList::new(entry.1, entry.0))
        .collect();
    results.sort();
    results
}

pub struct RawPropertyList {
    pub names: Vec<String>,
    pub values: Vec<String>,
}

impl RawPropertyList {
    pub fn new(mut names: Vec<String>, values: Vec<String>) -> Self {
        names.sort();
        RawPropertyList { names, values }
    }
}

impl Debug for RawPropertyList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} -> {:?}", &self.names, &self.values))
    }
}

impl PartialOrd for RawPropertyList {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.names.get(0).partial_cmp(&other.names.get(0))
    }
}

impl Ord for RawPropertyList {
    fn cmp(&self, other: &Self) -> Ordering {
        self.names.get(0).cmp(&other.names.get(0))
    }
}

impl PartialEq<Self> for RawPropertyList {
    fn eq(&self, other: &Self) -> bool {
        self.names.eq(&other.names) && self.values.eq(&other.values)
    }
}

impl Eq for RawPropertyList {}
