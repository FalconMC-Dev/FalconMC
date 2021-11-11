use crate::errors::*;

mod connection;

use connection::ClientConnection;
use falcon_core::ShutdownHandle;
use tokio::net::TcpListener;

pub struct NetworkListener {
    shutdown_handle: ShutdownHandle,
}

impl NetworkListener {
    pub async fn start_network_listening(shutdown_handle: ShutdownHandle) {
        info!("Starting network listening...");

        let network_listener = NetworkListener { shutdown_handle };

        network_listener.start_listening().await;
    }

    async fn start_listening(mut self) {
        let listener = match TcpListener::bind(("127.0.0.1", 30000))
            .await
            .chain_err(|| "Could not bind to port 30000!")
        {
            Ok(listener) => listener,
            Err(ref error) => {
                print_error!(error);
                return self.shutdown_handle.send_shutdown();
            }
        };

        loop {
            tokio::select! {
                _ = self.shutdown_handle.wait_for_shutdown() => {
                    break;
                }
                connection = listener.accept() => {
                    match connection {
                        Ok((socket, addr)) => {
                            debug!("Accepted connection at {}", &addr);
                            tokio::spawn(ClientConnection::process_socket(self.shutdown_handle.clone(), socket, addr));
                        },
                        Err(e) => {
                            print_error!(arbitrary_error!(e, ErrorKind::Msg(String::from("Connection broke!"))));
                        }
                    }
                }
            }
        }
        info!("Stopped network listening!");
    }
}
