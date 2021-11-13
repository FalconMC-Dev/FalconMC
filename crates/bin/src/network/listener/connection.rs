use crate::errors::*;
use std::io::Cursor;

use bytes::{Buf, BytesMut};
use crossbeam::channel::Sender;
use falcon_core::network::buffer::{ByteLimitCheck, PacketBufferRead};
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
                    match self.parse_frame() {
                        Ok(packet) => {
                            if let Some((preceding, len)) = packet {
                                if let Err(ref error) = self.handle_packet_buffer(preceding, len) { // TODO: disconnect
                                    print_error!(error);
                                    break;
                                }
                            }
                        },
                        Err(ref error) => { // TODO: communicate disconnect to main server
                            print_error!(error);
                            break;
                        }
                    }
                    debug!("Received {} bytes", n);
                }
            }
        }
    }

    fn handle_packet_buffer(&mut self, preceding: usize, len: usize) -> Result<()> {
        let _packet = self
            .buffer
            .split_to(preceding + len)
            .split_off(preceding)
            .freeze();
        // TODO: packet handling + plugin
        Ok(())
    }

    /// Reads a whole packet from the buffer and returns
    /// both the packet and the byte count with how many bytes it took to read the packet.
    ///
    /// (TODO: add compression and encryption mode!)
    ///
    /// Returns (preceding byte count, frame length)
    fn parse_frame(&self) -> Result<Option<(usize, usize)>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        let mut length_bytes: [u8; 3] = [0; 3];
        for i in 0..3 {
            if buf.remaining() == 0 {
                return Ok(None);
            }

            length_bytes[i] = buf.get_u8();

            if length_bytes[i] & 0b1000_0000 == 0 {
                let mut length_buf = Cursor::new(&length_bytes[..]);
                let frame_length = length_buf.read_var_i32()? as usize;
                trace!("Frame is {} bytes long", frame_length);
                buf.ensure_bytes_available(frame_length)?;
                return Ok(Some((i + 1, frame_length)));
            }
        }
        Err(ErrorKind::InvalidPacketLength.into())
    }
}
