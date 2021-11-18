use std::io::Cursor;
use std::net::SocketAddr;

use bytes::{Buf, BytesMut};
use crossbeam::channel::Sender;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use falcon_core::network::buffer::{ByteLimitCheck, PacketBufferRead, PacketBufferWrite};
use falcon_core::network::connection::{ConnectionTask, MinecraftConnection};
use falcon_core::network::packet::PacketEncode;
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
    out_buffer: BytesMut,
    in_buffer: BytesMut,
    server_tx: Sender<Box<McTask>>,
    output_sync: (UnboundedSender<BytesMut>, UnboundedReceiver<BytesMut>),
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
            out_buffer: BytesMut::with_capacity(4096),
            in_buffer: BytesMut::with_capacity(4096),
            server_tx,
            output_sync: tokio::sync::mpsc::unbounded_channel(),
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
                Some(packet) = self.output_sync.1.recv() => {
                    trace!("Outgoing length: {}", packet.len());
                    if let Err(ref e) = self.socket.write_all(packet.as_ref()).await.chain_err(|| "Error whilst sending packet") {
                        print_error!(e);
                    }
                }
                length = self.socket.read_buf(&mut self.in_buffer) => {
                    let n = match length {
                        Ok(n) => n,
                        Err(error) => {
                            print_error!(arbitrary_error!(error, ErrorKind::Msg(String::from("Error whilst receiving packet!"))));
                            // TODO: proper disconnect handling
                            break;
                        }
                    };
                    if n == 0 {
                        // TODO: fix disconnect handling here
                        // disconnect
                        break;
                    }

                    match self.read_packets() {
                        Err(ref e) => { // TODO: tell main server disconnect happened
                            print_error!(e);
                            break;
                        }
                        _ => {}
                    }
                    debug!("Received {} bytes, internal buffer size: {}", n, self.in_buffer.remaining());
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
            .in_buffer
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
        let mut buf = Cursor::new(&self.in_buffer[..]);
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

    fn send_packet(&mut self, packet_id: i32, packet_out: &dyn PacketEncode) {
        trace!("Sending packet!!! :D");
        self.out_buffer.write_var_i32(packet_id);
        packet_out.to_buf(&mut self.out_buffer);
        let temp_buf = self.out_buffer.split();
        self.out_buffer.write_var_i32(temp_buf.len() as i32);
        self.out_buffer.unsplit(temp_buf);
        if let Err(ref e) = self
            .output_sync
            .0
            .send(self.out_buffer.split())
            .chain_err(|| "Logically impossible error happened :)")
        {
            print_error!(e);
        }
    }

    fn disconnect(&mut self, _reason: String) {} // TODO: change into ChatComponent
}
