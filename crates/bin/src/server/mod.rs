use crate::errors::*;
use std::thread;

use crate::network::listener::NetworkListener;
use falcon_core::ShutdownHandle;

pub struct MainServer {
    shutdown_handle: ShutdownHandle,
}

impl MainServer {
    pub fn start_server(shutdown_handle: ShutdownHandle) -> Result<()> {
        info!("Starting server thread...");

        let server = MainServer { shutdown_handle };

        tokio::spawn(NetworkListener::start_network_listening(
            server.shutdown_handle.clone(),
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
