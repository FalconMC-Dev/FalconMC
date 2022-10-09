use std::fs::{File, OpenOptions};
use std::io::ErrorKind::NotFound;
use std::path::Path;

use anyhow::{Context, Error, Result};
use falcon_core::server::config::FalconConfig;
use falcon_core::ShutdownHandle;
use tracing::metadata::LevelFilter;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{reload, Layer};

mod error;
mod network;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    human_panic::setup_panic!();

    let log_file = load_log_file().context("Could not load log file")?;

    let (layer_file, handle_file) = reload::Layer::new(
        tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .with_writer(log_file)
            .with_filter(LevelFilter::DEBUG),
    );
    let (layer_stdout, handle_stdout) = reload::Layer::new(
        tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_writer(std::io::stdout)
            .with_filter(LevelFilter::INFO),
    );
    tracing_subscriber::registry().with(layer_file).with(layer_stdout).init();

    info!("Launching Falcon Server!");

    if let Err(e) = || -> Result<(), Error> {
        debug!("Loading config!");
        FalconConfig::init_config("config/falcon.toml").context(
            "The configuration file could not be loaded! This can most likely be solved by removing the config file and adjusting the config again after \
             having launched (and shut down) FalconMC.",
        )?;

        let filter_level = FalconConfig::global().tracing_level();
        handle_file.modify(|l| {
            *l.filter_mut() = filter_level;
        })?;
        handle_stdout.modify(|l| {
            *l.filter_mut() = filter_level;
        })?;
        Ok::<(), Error>(())
    }() {
        print_error!(e);
        return Ok(());
    }

    let (mut shutdown_handle, mut finished_rx) = ShutdownHandle::new();
    if let Err(e) = server::start_server(shutdown_handle.clone()) {
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

    drop(shutdown_handle);
    let _ = finished_rx.recv().await;
    info!("Falcon Server has shut down!");
    Ok(())
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
        },
        Err(e) => Err(e),
    }
}
