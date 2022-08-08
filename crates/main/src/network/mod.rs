use anyhow::{anyhow, Context};
use falcon_core::server::config::FalconConfig;
use falcon_core::ShutdownHandle;
use falcon_logic::connection::ConnectionReceiver;
use falcon_logic::server::ServerWrapper;
use falcon_logic::FalconConnection;
use ignore_result::Ignore;
use tokio::net::TcpListener;
use tracing::{info, debug};

pub struct NetworkListener {
    shutdown_handle: ShutdownHandle,
    /// Used to clone for every client handler per connection
    server: ServerWrapper,
}

impl NetworkListener {
    pub async fn start_network_listening(shutdown_handle: ShutdownHandle, server: ServerWrapper) {
        info!("Starting network listening...");
        debug!(
            "Connection size: {}",
            std::mem::size_of::<FalconConnection>()
        );

        let network_listener = NetworkListener {
            shutdown_handle,
            server,
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
                            let connection = FalconConnection::new(
                                self.shutdown_handle.clone(),
                                addr,
                                socket,
                                self.server.clone(),
                            ).await;
                            tokio::spawn(connection.start(FalconReceiver));
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

struct FalconReceiver;

impl ConnectionReceiver for FalconReceiver {
    fn receive(
        &mut self,
        packet_id: i32,
        bytes: &mut bytes::Bytes,
        connection: &mut FalconConnection,
    ) -> Result<Option<()>, falcon_core::error::FalconCoreError> {
        falcon_receive::falcon_process_packet(packet_id, bytes, connection)
    }
}
