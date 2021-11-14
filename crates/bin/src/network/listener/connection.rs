use std::io::Cursor;
use std::net::SocketAddr;

use bytes::{Buf, BytesMut};
use crossbeam::channel::Sender;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use falcon_core::network::buffer::{ByteLimitCheck, PacketBufferRead};
use falcon_core::network::connection::{ConnectionTask, MinecraftConnection};
use falcon_core::network::PacketHandlerState;
use falcon_core::server::McTask;
use falcon_core::ShutdownHandle;
use falcon_protocol::UNKNOWN_PROTOCOL;

use crate::errors::*;

pub struct ClientConnection {
    shutdown_handle: ShutdownHandle,
    socket: TcpStream,
    addr: SocketAddr,
    // packet handling
    handler_state: PacketHandlerState,
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
            handler_state: PacketHandlerState::new(UNKNOWN_PROTOCOL),
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
                Some(task) = self.connection_sync.1.recv() => {
                    task(&mut self);
                }
                length = self.socket.read_buf(&mut self.buffer) => {
                    let n = match length {
                        Ok(n) => n,
                        Err(error) => {
                            print_error!(arbitrary_error!(error, ErrorKind::Msg(String::from("Error whilst receiving packet!"))));
                            // TODO: proper disconnect handling
                            break;
                        }
                    };
                    if n == 0 {
                        // TODO: fix handling here
                        // disconnect
                        break;
                    }

                    // TODO: read packets
                    match self.read_packets() {
                        Err(ref e) => { // TODO: tell main server disconnect happened
                            print_error!(e);
                            break;
                        }
                        _ => {}
                    }
                    debug!("Received {} bytes, internal buffer size: {}", n, self.buffer.remaining());
                }
            }
        }
    }

    fn read_packets(&mut self) -> Result<()> {
        while let Some((preceding, len)) = self.parse_frame()? {
            self.handle_packet_buffer(preceding, len)?;
        }
        Ok(())
    }

    fn handle_packet_buffer(&mut self, preceding: usize, len: usize) -> Result<()> {
        let mut packet = self
            .buffer
            .split_to(preceding + len)
            .split_off(preceding)
            .freeze();
        let packet_id = packet.read_var_i32()?;
        debug!("Packet id = {}", packet_id);
        if let None = falcon_protocol::manager::PROTOCOL_MANAGER.process_packet(
            packet_id,
            &mut packet,
            self,
        )? {
            debug!("Unknown packet received, skipping!");
        }
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

impl MinecraftConnection for ClientConnection {
    fn get_handler_state(&self) -> &PacketHandlerState {
        &self.handler_state
    }

    fn get_handler_state_mut(&mut self) -> &mut PacketHandlerState {
        &mut self.handler_state
    }

    fn get_server_link_mut(&mut self) -> &mut Sender<Box<McTask>> {
        &mut self.server_tx
    }

    fn disconnect(&mut self, _reason: String) {} // TODO: change into ChatComponent
}
