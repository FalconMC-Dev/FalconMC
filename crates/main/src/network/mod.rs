mod connection;

use anyhow::Context;
use ignore_result::Ignore;
use falcon_core::network::connection::ClientConnection;
use falcon_core::server::config::FalconConfig;
use falcon_core::server::McTask;
use falcon_core::ShutdownHandle;
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;

pub struct NetworkListener {
    shutdown_handle: ShutdownHandle,
    /// Used to clone for every client handler per connection
    server_tx: UnboundedSender<Box<McTask>>,
}

impl NetworkListener {
    pub async fn start_network_listening(
        shutdown_handle: ShutdownHandle,
        server_tx: UnboundedSender<Box<McTask>>,
    ) {
        info!("Starting network listening...");
        debug!("Connection size: {}", std::mem::size_of::<ClientConnection>());

        let network_listener = NetworkListener {
            shutdown_handle,
            server_tx,
        };

        network_listener.start_listening().await;
    }

    #[tracing::instrument(name = "network", skip(self))]
    async fn start_listening(mut self) {
        let listener = match TcpListener::bind(FalconConfig::global().server_socket_addrs())
            .await
            .with_context(|| "Could not bind to the address!")
        {
            Ok(listener) => listener,
            Err(ref error) => {
                print_error!(error);
                return self.shutdown_handle.send_shutdown();
            }
        };
        info!("Network bound to {}", listener.local_addr().unwrap());

        loop {
            tokio::select! {
                _ = self.shutdown_handle.wait_for_shutdown() => {
                    break;
                }
                connection = listener.accept() => {
                    match connection {
                        Ok((socket, addr)) => {
                            debug!(address = %addr, "Accepted connection");
                            socket.set_nodelay(true).ignore();
                            tokio::spawn(connection::new_connection(self.shutdown_handle.clone(), socket, addr, self.server_tx.clone()));
                        },
                        Err(e) => {
                            print_error!(anyhow!("Connection broke due to {}", e));
                        }
                    }
                }
            }
        }
        info!("Stopped network listening!");
    }
}
