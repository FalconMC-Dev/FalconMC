use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use linked_hash_map::LinkedHashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RawBlockData {
    pub properties: Option<LinkedHashMap<String, Vec<String>>>,
    pub states: Vec<RawBlockState>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawBlockState {
    pub properties: Option<LinkedHashMap<String, String>>,
    pub id: i32,
    pub default: Option<bool>,
}

pub struct RawPropertyList {
    pub names: Vec<String>,
    pub values: Vec<String>,
}

impl RawPropertyList {
    pub fn new(mut names: Vec<String>, values: Vec<String>) -> Self {
        names.sort();
        RawPropertyList {
            names,
            values,
        }
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

pub fn collect_properties(data: &LinkedHashMap<String, RawBlockData>) -> Vec<RawPropertyList> {
    let mut property_values = HashMap::new();
    for (_, entry) in data.iter() {
        if let Some(ref map) = entry.properties {
            for (name, values) in map.iter() {
                if !property_values.contains_key(values) {
                    let mut list = Vec::new();
                    list.push(name.clone());
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