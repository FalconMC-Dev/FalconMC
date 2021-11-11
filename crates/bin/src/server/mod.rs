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

        let shutdown_handle2 = server.shutdown_handle.clone();
        thread::Builder::new()
            .name(String::from("Main Server Thread"))
            .spawn(|| server.start_server_logic())
            .chain_err(|| "Couldn't start server logic!")?;

        tokio::spawn(NetworkListener::start_network_listening(shutdown_handle2));

        Ok(())
    }

    fn start_server_logic(mut self) {}
}
