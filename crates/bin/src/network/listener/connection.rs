use crate::errors::*;

use bytes::BytesMut;
use crossbeam::channel::Sender;
use falcon_core::network::connection::ConnectionTask;
use falcon_core::server::McTask;
use falcon_core::ShutdownHandle;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct ClientConnection {
    shutdown_handle: ShutdownHandle,
    socket: TcpStream,
    addr: SocketAddr,
    // packet handling
    buffer: BytesMut,
    server_tx: Sender<Box<McTask>>,
    connection_sync: (
        UnboundedSender<Box<ConnectionTask>>,
        UnboundedReceiver<Box<ConnectionTask>>,
    ),
}

impl ClientConnection {
    pub(crate) async fn process_socket(
        shutdown_handle: ShutdownHandle,
        socket: TcpStream,
        addr: SocketAddr,
        server_tx: Sender<Box<McTask>>,
    ) {
        let connection = ClientConnection {
            shutdown_handle,
            socket,
            addr,
            buffer: BytesMut::with_capacity(4096),
            server_tx,
            connection_sync: tokio::sync::mpsc::unbounded_channel(),
        };

        connection.start_packet_loop().await;
    }

    async fn start_packet_loop(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_handle.wait_for_shutdown() => {
                    break;
                }
                Ok(n) = self.socket.read_buf(&mut self.buffer) => {
                    if n == 0 {
                        // TODO: fix handling here
                        // disconnect
                        break;
                    }

                    // TODO: read packets

                    debug!("Received {} bytes", n);
                }
            }
        }
    }
}
