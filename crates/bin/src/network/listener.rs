use tokio::net::TcpListener;
use falcon_core::ShutdownHandle;

pub struct NetworkListener {
    shutdown_handle: ShutdownHandle,
}

impl NetworkListener {
    pub async fn start_network_listening(shutdown_handle: ShutdownHandle) {
        info!("Starting network listening...");

        let network_listener = NetworkListener {
            shutdown_handle
        };

        network_listener.start_listening().await;
    }

    async fn start_listening(mut self) {
        let listener = match TcpListener::bind(("127.0.0.1", 30000)).await {
            Ok(listener) => listener,
            Err(error) => {
                error!("Could not bind to port due to \"{}\"", error);
                return self.shutdown_handle.send_shutdown();
            }
        };

        loop {
            tokio::select! {
                _ = self.shutdown_handle.wait_for_shutdown() => {
                    break;
                }
                connection = listener.accept() => {
                    if let Ok((_socket, addr)) = connection {
                        debug!("Accepted connection at {}", &addr);
                    }
                }
            }
        }
        info!("Stopped network listening!");
    }
}