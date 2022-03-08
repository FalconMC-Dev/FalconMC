#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate tracing;

use std::fs::{File, OpenOptions};
use std::io::ErrorKind::NotFound;
use std::path::Path;
use tracing::metadata::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use falcon_core::server::config::FalconConfig;
use falcon_core::ShutdownHandle;

use crate::errors::*;
use crate::player::Player;
use crate::server::MainServer;

mod errors;
mod network;
mod server;
mod player;

#[tokio::main]
async fn main() {
    // TODO: Link config to logging level
    let log_file = load_log_file().unwrap();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .with_writer(log_file)
            .with_filter(LevelFilter::DEBUG))
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_writer(std::io::stdout)
            .with_filter(LevelFilter::DEBUG))
        .init();

    info!("Launching Falcon Server!");
    debug!("Player size: {}", std::mem::size_of::<Player>());

    debug!("Loading config!");
    if let Err(ref e) = FalconConfig::init_config("config/falcon.toml")
        .chain_err(|| "The configuration file could not be loaded!")
    {
        print_error!(e);
        return;
    }

    let (mut shutdown_handle, mut finished_rx) = ShutdownHandle::new();
    if let Err(ref e) = MainServer::start_server(shutdown_handle.clone()) {
        print_error!(e);
        shutdown_handle.send_shutdown();
    } else {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down server! (Ctrl-C exited)");
                shutdown_handle.send_shutdown();
            }
            _ = shutdown_handle.wait_for_shutdown() => {}
        }
    }

    std::mem::drop(shutdown_handle);
    let _ = finished_rx.recv().await;
    info!("Falcon Server has shut down!");
}

fn load_log_file() -> std::io::Result<File> {
    let path = Path::new("./logs/debug.log");
    match OpenOptions::new().append(true).create(true).open("./logs/debug.log") {
        Ok(log_file) => Ok(log_file),
        Err(ref e) if e.kind() == NotFound => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            OpenOptions::new().append(true).create(true).open("./logs/debug.log")
        }
        Err(e) => Err(e),
    }
}