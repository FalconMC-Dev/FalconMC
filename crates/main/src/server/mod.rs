use std::io::Read;
use std::thread;

use anyhow::{Context, Result};
use falcon_core::server::config::FalconConfig;
use falcon_logic::server::ServerWrapper;
use flate2::read::GzDecoder;
use tokio::sync::mpsc::unbounded_channel;

use falcon_core::schematic::{SchematicData, SchematicVersionedRaw};
use falcon_core::ShutdownHandle;
use falcon_logic::{FalconServer, FalconWorld};

use tracing::{info, debug};

use crate::network::NetworkListener;
use crate::server::console::ConsoleListener;

pub mod console;

pub(crate) fn start_server(shutdown_handle: ShutdownHandle) -> Result<()> {
    info!("Starting server thread...");

    let world = match FalconConfig::global().world_file() {
        Some(world_file) => { 
            let world_file = std::fs::read(world_file)
                .with_context(|| "Could not load world schematic, stopping launch")?;
            let mut gz = GzDecoder::new(&world_file[..]);
            let mut decompressed_world = Vec::new();
            gz.read_to_end(&mut decompressed_world)
                .with_context(|| "Could not decompress world.schem, is this a valid schematic?")?;
            debug!("Checkpoint - loaded schem file");
            let schematic: SchematicVersionedRaw = fastnbt::from_bytes(&decompressed_world)
                .with_context(|| "Could not parse schematic file, is this valid nbt?")?;
            debug!("Checkpoint - deserialized into raw format");
            let data = SchematicData::try_from(schematic).with_context(|| {
                "Invalid schematic, this server cannot use this schematic currently!"
            })?;
            
            debug!("Checkpoint - parsed raw format");
            info!("Loaded world");

            FalconWorld::try_from(data)?
        }
        None => FalconWorld::new(0, 0, 0, 0, 0),
    };

    let console_rx = ConsoleListener::start_console(shutdown_handle.clone())?;
    let (server_tx, server_rx) = unbounded_channel();
    let mut server = FalconServer::new(shutdown_handle, console_rx, server_rx, world);

    tokio::spawn(NetworkListener::start_network_listening(
        server.shutdown_handle().clone(),
        ServerWrapper::new(server_tx),
    ));

    thread::Builder::new()
        .name(String::from("Main Server Thread"))
        .spawn(move || server.start())
        .with_context(|| "Couldn't start server logic!")?;

    Ok(())
}
