use linked_hash_map::LinkedHashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawBlockData {
    properties: Option<LinkedHashMap<String, Vec<String>>>,
    states: Vec<RawBlockState>,
}

impl RawBlockData {
    pub fn base_id(&self) -> i32 {
        self.states.get(0).expect("Block without base id -> should be impossible!").id()
    }

    pub fn base_state(&self) -> &RawBlockState {
        for state in &self.states {
            if state.default.is_some() {
                return state;
            }
        }
        panic!("Expected a default state!");
    }

    pub fn properties(&self) -> Option<&LinkedHashMap<String, Vec<String>>> {
        self.properties.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct RawBlockState {
    properties: Option<LinkedHashMap<String, String>>,
    id: i32,
    default: Option<bool>,
}

impl RawBlockState {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn properties(&self) -> Option<&LinkedHashMap<String, String>> {
        self.properties.as_ref()
    }
}