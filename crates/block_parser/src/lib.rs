use std::fs;
use std::fs::read_dir;
use std::path::PathBuf;

use linked_hash_map::LinkedHashMap;

use crate::util::convert::into_block_data;
use crate::util::create_default_property_base;
use crate::util::data::BlockData;
use crate::util::processing::BlockList;
use crate::util::properties::EnumPropertyBase;
use crate::util::raw::RawBlockData;

pub mod util;

pub fn load_block_lists() -> (Vec<BlockList>, EnumPropertyBase) {
    let mut data_files: Vec<(i32, LinkedHashMap<String, RawBlockData>)> = find_data_files().iter()
        .map(|(version, filename)| (*version, get_data(filename))).collect();

    let mut property_base_set = create_default_property_base();

    (data_files.drain(..).enumerate().map(|(i, (data_version, raw_data))| {
        BlockList::new(data_version, raw_data.iter().map(|(name, raw)| (name.clone(), into_block_data(raw, &mut property_base_set, i == 0))).collect())
    }).collect(), property_base_set)
}

pub fn find_data_files() -> Vec<(i32, String)> {
    let files = read_dir(String::from(env!("CARGO_MANIFEST_DIR")) + "/raw_data").unwrap();
    let mut output = Vec::new();
    for file in files {
        let file = file.unwrap();
        let name = file.file_name().into_string().unwrap();
        // data file names should be "blocks-***.json" with "***" being the data version
        let (check, trimmed_front) = name.split_at(7);
        if check == "blocks-" {
            let trimmed = trimmed_front.split_at(trimmed_front.len() - 5).0;
            output.push((trimmed.parse::<i32>().unwrap(), name));
        }
    }
    output.sort_by(|x1, x2| x2.0.cmp(&x1.0));
    output
}

pub fn get_data(filename: &str) -> LinkedHashMap<String, RawBlockData> {
    let mut work_dir = PathBuf::from(String::from(env!("CARGO_MANIFEST_DIR")) + "/raw_data");
    work_dir.push(filename);
    serde_json::from_str(&fs::read_to_string(work_dir).unwrap()).unwrap()
}



