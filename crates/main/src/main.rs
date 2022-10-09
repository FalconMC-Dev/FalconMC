use std::fs::{File, OpenOptions};
use std::io::ErrorKind::NotFound;
use std::path::Path;

use anyhow::Context;
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
async fn main() {
    human_panic::setup_panic!();

    let log_file = match load_log_file().context("Could not load config file") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", e);
            return;
        },
    };

    let (layer_file, handle_file) = reload::Layer::new(
        tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .with_writer(log_file)
            .with_filter(LevelFilter::TRACE),
    );
    let (layer_stdout, handle_stdout) = reload::Layer::new(
        tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_writer(std::io::stdout)
            .with_filter(LevelFilter::TRACE),
    );

    tracing_subscriber::registry().with(layer_file).with(layer_stdout).init();

    info!("Launching Falcon Server!");

    debug!("Loading config!");
    if let Err(e) = FalconConfig::init_config("config/falcon.toml").context(
        "The configuration file could not be loaded! This can most likely be solved by removing the config file and adjusting the config again after having \
         launched (and shut down) FalconMC.",
    ) {
        print_error!(e);
        return;
    }

    let filter_level = match FalconConfig::global()
        .tracing_level()
        .context("Possible tracing levels are tracing, debug, info, warn, error.")
    {
        Ok(val) => val,
        Err(e) => {
            print_error!(e);
            return;
        },
    };
    handle_file
        .modify(|l| {
            *l.filter_mut() = filter_level;
        })
        .unwrap();
    handle_stdout
        .modify(|l| {
            *l.filter_mut() = filter_level;
        })
        .unwrap();

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
