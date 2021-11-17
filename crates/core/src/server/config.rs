use confy::ConfyError;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static INSTANCE: OnceCell<FalconConfig> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalconConfig {
    max_players: i32,
    server_port: u16,
    server_ip: String,
    allow_flight: bool,
}

impl Default for FalconConfig {
    fn default() -> Self {
        FalconConfig {
            max_players: -1,
            server_port: 30000,
            server_ip: String::from("0.0.0.0"),
            allow_flight: false,
        }
    }
}

impl FalconConfig {
    pub fn global() -> &'static FalconConfig {
        INSTANCE.get().expect("FalconConfig is not initialized!!")
    }

    pub fn init_config(name: &str) -> Result<(), ConfyError> {
        INSTANCE.set(confy::load_path(name)?).unwrap();
        Ok(())
    }

    pub fn max_players(&self) -> i32 {
        self.max_players
    }
    pub fn server_port(&self) -> u16 {
        self.server_port
    }
    pub fn server_ip(&self) -> &str {
        &self.server_ip
    }
    pub fn allow_flight(&self) -> bool {
        self.allow_flight
    }
}
