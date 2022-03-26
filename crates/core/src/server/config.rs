use crate::player::{LookAngles, Position};
use confy::ConfyError;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use tokio::net::ToSocketAddrs;

static INSTANCE: OnceCell<FalconConfig> = OnceCell::new();

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FalconConfig {
    connection: ConnectionConfig,
    players: PlayerSettings,
    server: ServerSettings,

}

impl FalconConfig {
    pub fn global() -> &'static FalconConfig {
        INSTANCE.get().expect("FalconConfig is not initialized!!")
    }

    pub fn init_config(name: &str) -> Result<(), ConfyError> {
        INSTANCE.set(confy::load_path(name)?).unwrap();
        Ok(())
    }

    pub fn server_port(&self) -> u16 {
        self.connection.server_port
    }

    pub fn server_ip(&self) -> IpAddr {
        self.connection.server_ip
    }

    pub fn max_players(&self) -> i32 {
        self.server.max_players
    }

    pub fn description(&self) -> &str {
        &self.server.description
    }

    pub fn allow_flight(&self) -> bool {
        self.players.allow_flight
    }

    pub fn max_view_distance(&self) -> u8 {
        self.players.max_view_distance
    }

    pub fn spawn_pos(&self) -> Position {
        self.players.spawn_position
    }

    pub fn spawn_look(&self) -> LookAngles {
        self.players.spawn_look
    }

    pub fn server_socket_addrs(&self) -> impl ToSocketAddrs + '_ {
        (self.server_ip(), self.server_port())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionConfig {
    server_ip: IpAddr,
    server_port: u16,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        ConnectionConfig {
            server_port: 30000,
            server_ip: IpAddr::from_str("0.0.0.0").unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSettings {
    allow_flight: bool,
    max_view_distance: u8,
    spawn_position: Position,
    spawn_look: LookAngles,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        PlayerSettings {
            allow_flight: false,
            max_view_distance: 10,
            spawn_position: Default::default(),
            spawn_look: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerSettings {
    max_players: i32,
    description: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        ServerSettings {
            max_players: -1,
            description: String::from("§eFalcon server§r§b!!!"),
        }
    }
}
