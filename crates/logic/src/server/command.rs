use std::collections::VecDeque;
use std::str::FromStr;

use thiserror::Error;

#[derive(PartialEq, Eq, Debug)]
pub enum Command {
    Stop,
    Kick(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CommandParseError {
    #[error("Input was empty")]
    EmptyInput,
    #[error("Erroneous backslash or ended inside open quotation")]
    ErroneousInput,
    #[error("Unknown command {0}")]
    InvalidCommand(String),
    /// Contains syntax explanation string
    #[error("Not enough arguments. {0}")]
    NotEnoughArgs(&'static str),
}

impl FromStr for Command {
    type Err = CommandParseError;

    /// Parse command and it's arguments. Does not care about extra arguments
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: `vec.into::<VecDeque<_>>` is not preferable
        // Use `shlex::Shlex::new` to get an iterator to collect into `VecDeque` rather
        // than using the `Vec` intermediary Problem is you need check
        // `Shlex.had_error` AFTER it's done iterating, so you can't use `collect` as
        // that consumes shlex
        let mut split: VecDeque<_> = match shlex::split(s.trim()) {
            Some(val) => val.into(),
            None => return Err(CommandParseError::ErroneousInput),
        };

        let command = match split.pop_front() {
            Some(val) => val,
            None => return Err(CommandParseError::EmptyInput),
        }
        .to_lowercase();
        match command.as_str() {
            "stop" => Ok(Self::Stop),
            "kick" => match split.pop_front().map(Command::Kick) {
                Some(val) => Ok(val),
                None => Err(CommandParseError::NotEnoughArgs("kick <username>")),
            },
            _ => Err(CommandParseError::InvalidCommand(command)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kick() {
        let input = r#"kick "cool PLAYER""#;
        let res = Command::from_str(input).unwrap();
        assert_eq!(Command::Kick(String::from("cool PLAYER")), res);
    }

    #[test]
    fn stop() {
        let input = "stop";
        let res = Command::from_str(input).unwrap();
        assert_eq!(Command::Stop, res);
    }

    #[test]
    fn erroneous_input() {
        let input = r#"42"\"#;
        let res = Command::from_str(input);
        assert_eq!(Err(CommandParseError::ErroneousInput), res);
    }

    #[test]
    fn invalid_command() {
        let input = "";
        let res = Command::from_str(input);
        assert_eq!(Err(CommandParseError::EmptyInput), res);
    }
}
