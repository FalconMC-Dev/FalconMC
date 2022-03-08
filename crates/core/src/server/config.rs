use crate::player::{LookAngles, Position};
use confy::ConfyError;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use tokio::net::ToSocketAddrs;

static INSTANCE: OnceCell<FalconConfig> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalconConfig {
    max_players: i32,
    server_port: u16,
    server_ip: IpAddr,
    allow_flight: bool,
    max_view_distance: u8,
    spawn_position: Position,
    spawn_look: LookAngles,
}

impl Default for FalconConfig {
    fn default() -> Self {
        FalconConfig {
            max_players: -1,
            server_port: 30000,
            server_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            allow_flight: false,
            max_view_distance: 10,
            spawn_position: Default::default(),
            spawn_look: Default::default(),
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
    pub fn server_ip(&self) -> IpAddr {
        self.server_ip
    }
    pub fn allow_flight(&self) -> bool {
        self.allow_flight
    }

    pub fn max_view_distance(&self) -> u8 {
        self.max_view_distance
    }

    pub fn spawn_pos(&self) -> Position {
        self.spawn_position
    }

    pub fn spawn_look(&self) -> LookAngles {
        self.spawn_look
    }

    pub fn server_socket_addrs(&self) -> impl ToSocketAddrs + '_ {
        (self.server_ip(), self.server_port())
    }
}
