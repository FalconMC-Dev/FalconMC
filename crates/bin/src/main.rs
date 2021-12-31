#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate tracing;

use tracing::Level;
use tracing_subscriber::EnvFilter;
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
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_ansi(false)
        .with_target(false)
        .with_max_level(Level::INFO)
        .with_env_filter(EnvFilter::from_default_env())
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