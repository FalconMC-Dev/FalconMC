//! FalconMC base config module.
//!
//! # Note
//! This module may grow eventually, there are
//! no statements about that yet, though.

use serde::{Deserialize, Serialize};
use tracing::metadata::LevelFilter;

/// Base config struct.
///
/// This config struct is as general as possible. It is meant
/// to be used in higher crates and for that reason only provides
/// one default option for the log level.
#[derive(Debug, Serialize, Deserialize)]
pub struct FalconConfig<T> {
    #[serde(with = "tracing_serde")]
    pub tracing_level: LevelFilter,
    #[serde(flatten)]
    pub inner: T,
}

impl<T: Default> Default for FalconConfig<T> {
    fn default() -> Self {
        Self {
            tracing_level: LevelFilter::INFO,
            inner: T::default(),
        }
    }
}

mod tracing_serde {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};
    use tracing::metadata::LevelFilter;

    pub fn serialize<S: Serializer>(level: &LevelFilter, serializer: S) -> Result<S::Ok, S::Error> {
        match *level {
            LevelFilter::OFF => serializer.serialize_str("off"),
            LevelFilter::ERROR => serializer.serialize_str("error"),
            LevelFilter::WARN => serializer.serialize_str("warn"),
            LevelFilter::INFO => serializer.serialize_str("info"),
            LevelFilter::DEBUG => serializer.serialize_str("debug"),
            LevelFilter::TRACE => serializer.serialize_str("trace"),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<LevelFilter, D::Error> {
        let input: &'de str = <&'de str>::deserialize(deserializer)?;
        match input.trim().to_lowercase().as_str() {
            "off" => Ok(LevelFilter::OFF),
            "error" => Ok(LevelFilter::ERROR),
            "warn" => Ok(LevelFilter::WARN),
            "info" => Ok(LevelFilter::INFO),
            "debug" => Ok(LevelFilter::DEBUG),
            "trace" => Ok(LevelFilter::TRACE),
            _ => Err(Error::unknown_variant(input, &["off", "error", "warn", "info", "debug", "trace"])),
        }
    }
}
