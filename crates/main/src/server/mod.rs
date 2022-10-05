use std::io::Read;
use std::thread;

use anyhow::{Context, Result};
use falcon_core::schematic::{SchematicData, SchematicVersionedRaw};
use falcon_core::server::config::FalconConfig;
use falcon_core::ShutdownHandle;
use falcon_logic::server::ServerWrapper;
use falcon_logic::{FalconServer, FalconWorld};
use flate2::read::GzDecoder;
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;

use crate::network::NetworkListener;
use crate::server::console::ConsoleListener;

pub mod console;

pub(crate) fn start_server(shutdown_handle: ShutdownHandle) -> Result<()> {
    info!("Starting server thread...");

    let world = match FalconConfig::global().world_file() {
        Some(file_name) => {
            let world_file = std::fs::read(file_name).with_context(|| format!("Could not load \"{}\", stopping launch", file_name))?;
            let mut gz = GzDecoder::new(&world_file[..]);
            let mut decompressed_world = Vec::new();
            gz.read_to_end(&mut decompressed_world)
                .with_context(|| format!("Could not decompress \"{}\", is this a valid schematic?", file_name))?;

            let schematic: SchematicVersionedRaw =
                fastnbt::from_bytes(&decompressed_world).with_context(|| format!("Could not parse schematic file \"{}\", is this valid nbt?", file_name))?;

            let data = SchematicData::try_from(schematic)
                .with_context(|| format!("Invalid schematic, this server cannot use schematic \"{}\" currently!", file_name))?;

            info!("Loaded world");

            FalconWorld::try_from(data)?
        },
        None => FalconWorld::new(0, 0, 0, 0, 0),
    };

    let console_rx = ConsoleListener::start_console(shutdown_handle.clone())?;
    let (server_tx, server_rx) = unbounded_channel();
    let mut server = FalconServer::new(shutdown_handle, console_rx, server_rx, world);

    tokio::spawn(NetworkListener::start_network_listening(server.shutdown_handle().clone(), ServerWrapper::new(server_tx)));

    thread::Builder::new()
        .name(String::from("Main Server Thread"))
        .spawn(move || server.start())
        .with_context(|| "Couldn't start server logic!")?;

    Ok(())
}
