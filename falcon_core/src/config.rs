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
///
/// The log level is parsed by [`tracing`].
/// See `FromStr` in [`LevelFilter`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FalconConfig<T> {
    #[serde(with = "tracing_serde")]
    pub tracing_level: LevelFilter,
    #[serde(flatten)]
    pub inner: T,
}

impl<T: Default> Default for FalconConfig<T> {
    /// Default tracing level is [`LevelFilter::INFO`].
    fn default() -> Self {
        Self {
            tracing_level: LevelFilter::INFO,
            inner: T::default(),
        }
    }
}

mod tracing_serde {
    use std::str::FromStr;

    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};
    use tracing::metadata::LevelFilter;

    pub fn serialize<S: Serializer>(level: &LevelFilter, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(level.to_string().as_str())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<LevelFilter, D::Error> {
        let input: &'de str = <&'de str>::deserialize(deserializer)?;
        LevelFilter::from_str(input.trim().to_lowercase().as_str())
            .map_err(|_| Error::unknown_variant(input, &["off", "error", "warn", "info", "debug", "trace"]))
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

    use super::*;

    #[rustfmt::skip]
    #[test]
    fn test_config() {
        let mut config = FalconConfig::<()>::default();
        assert_tokens(&config, &[
            Token::Map { len: None },
            Token::Str("tracing_level"),
            Token::BorrowedStr("info"),
            Token::MapEnd,
        ]);

        config.tracing_level = LevelFilter::ERROR;
        assert_tokens(&config, &[
            Token::Map { len: None },
            Token::Str("tracing_level"),
            Token::BorrowedStr("error"),
            Token::MapEnd,
        ]);

        config.tracing_level = LevelFilter::DEBUG;
        assert_de_tokens(&config, &[
            Token::Map { len: None },
            Token::Str("tracing_level"),
            Token::BorrowedStr("4"),
            Token::MapEnd,
        ]);
        config.tracing_level = LevelFilter::ERROR;
        assert_de_tokens(&config, &[
            Token::Map { len: None },
            Token::Str("tracing_level"),
            Token::BorrowedStr(""),
            Token::MapEnd,
        ]);
        assert_de_tokens_error::<FalconConfig<()>>(&[
            Token::Map { len: None },
            Token::Str("tracing_level"),
            Token::BorrowedStr("invalid"),
            Token::MapEnd,
        ], "unknown variant `invalid`, expected one of `off`, `error`, `warn`, `info`, `debug`, `trace`");
    }
}
