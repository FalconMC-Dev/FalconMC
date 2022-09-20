use std::borrow::Cow;

use serde::Serialize;

use crate::server::config::FalconConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Peaceful = 0,
    Easy,
    Normal,
    Hard,
}

impl From<Difficulty> for u8 {
    fn from(src: Difficulty) -> Self {
        src as u8
    }
}

#[derive(Debug, Serialize)]
pub struct ServerVersion {
    pub name: Cow<'static, str>,
    pub protocol: i32,
}

impl ServerVersion {
    pub fn new<T: Into<Cow<'static, str>>>(name: T, protocol_id: i32) -> Self {
        let excluded = FalconConfig::global().excluded_versions();
        let (name, version) = if !FalconConfig::ALLOWED_VERSIONS.contains(&protocol_id.unsigned_abs()) || excluded.contains(&protocol_id.unsigned_abs()) {
            let (name, mut protocol) = ("Unsupported version".into(), FalconConfig::ALLOWED_VERSIONS[0]);
            for version in FalconConfig::ALLOWED_VERSIONS {
                if !excluded.contains(&version) {
                    protocol = version;
                }
            }
            (name, protocol as i32)
        } else {
            (name.into(), protocol_id)
        };
        ServerVersion {
            name,
            protocol: version,
        }
    }
}
