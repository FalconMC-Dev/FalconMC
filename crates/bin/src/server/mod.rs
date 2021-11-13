use crate::errors::*;
use crossbeam::channel::Receiver;
use falcon_core::server::McTask;
use std::thread;

use crate::network::listener::NetworkListener;
use falcon_core::ShutdownHandle;

pub struct MainServer {
    shutdown_handle: ShutdownHandle,
    server_rx: Receiver<Box<McTask>>,
}

impl MainServer {
    pub fn start_server(shutdown_handle: ShutdownHandle) -> Result<()> {
        info!("Starting server thread...");

        let (server_tx, server_rx) = crossbeam::channel::unbounded();
        let server = MainServer {
            shutdown_handle,
            server_rx,
        };

        tokio::spawn(NetworkListener::start_network_listening(
            server.shutdown_handle.clone(),
            server_tx,
        ));

        thread::Builder::new()
            .name(String::from("Main Server Thread"))
            .spawn(|| server.start_server_logic())
            .chain_err(|| "Couldn't start server logic!")?;

        Ok(())
    }

    fn start_server_logic(mut self) {
        // TODO: ticking etc.
    }
}
