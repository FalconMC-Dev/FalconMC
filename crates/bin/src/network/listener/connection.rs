use std::io::Cursor;
use std::net::SocketAddr;
use std::time::Duration;

use bytes::{Buf, BytesMut};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::{interval, Interval, MissedTickBehavior};

use falcon_core::network::buffer::{ByteLimitCheck, PacketBufferRead, PacketBufferWrite};
use falcon_core::network::connection::{ConnectionTask, MinecraftConnection};
use falcon_core::network::packet::PacketEncode;
use falcon_core::network::ConnectionState::{Login, Status};
use falcon_core::network::{ConnectionState, PacketHandlerState};
use falcon_core::server::McTask;
use falcon_core::ShutdownHandle;
use falcon_protocol::UNKNOWN_PROTOCOL;

use crate::errors::*;

pub struct ClientConnection {
    shutdown_handle: ShutdownHandle,
    // connection data
    socket: TcpStream,
    addr: SocketAddr,
    // packet handling
    handler_state: PacketHandlerState,
    out_buffer: BytesMut,
    in_buffer: BytesMut,
    // synchronization
    time_out: Interval,
    server_tx: UnboundedSender<Box<McTask>>,
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
        server_tx: UnboundedSender<Box<McTask>>,
    ) {
        let mut time_out = interval(Duration::from_secs(30));
        time_out.set_missed_tick_behavior(MissedTickBehavior::Delay);
        time_out.tick().await;
        let connection = ClientConnection {
            shutdown_handle,
            socket,
            addr,
            handler_state: PacketHandlerState::new(UNKNOWN_PROTOCOL),
            out_buffer: BytesMut::with_capacity(4096),
            in_buffer: BytesMut::with_capacity(4096),
            time_out,
            server_tx,
            connection_sync: tokio::sync::mpsc::unbounded_channel(),
        };

        connection.start_packet_loop().await;
    }

    #[tracing::instrument(name = "client", skip(self), fields(address = %self.addr))]
    async fn start_packet_loop(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_handle.wait_for_shutdown() => {
                    break;
                }
                _ = self.time_out.tick() => {
                    self.disconnect(String::from("Did not receive Keep alive packet!"));
                }
                readable = self.socket.readable(), if self.handler_state.connection_state() != ConnectionState::Disconnected => {
                    let span = trace_span!("incoming_data", state = %self.handler_state);
                    let _enter = span.enter();
                    if let Err(e) = readable {
                        error!("Error on read: {}", e);
                        self.handler_state.set_connection_state(ConnectionState::Disconnected);
                        break;
                    }
                    match self.socket.try_read_buf(&mut self.in_buffer) {
                        Ok(0) => {
                            trace!("Connection lost!");
                            self.handler_state.set_connection_state(ConnectionState::Disconnected);
                            break;
                        }
                        Ok(n) => {
                            trace!(length = n, "Data received!");
                            if let Err(e) = self.read_packets() {
                                debug!("Read error: {}", e);
                                self.disconnect(String::from("Error while reading packet"));
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            if self.handler_state.connection_state() == ConnectionState::Play {
                                error!("Unexpected error ocurred on read: {}", e);
                            }
                            self.disconnect(format!("Unexpected error ocurred on read: {}", e));
                        }
                    }
                }
                writable = self.socket.writable(), if !self.out_buffer.is_empty() => {
                    let span = trace_span!("outgoing_data", state = %self.handler_state);
                    let _enter = span.enter();
                    if let Err(e) = writable {
                        error!("Error on write: {}", e);
                        self.handler_state.set_connection_state(ConnectionState::Disconnected);
                        break;
                    }
                    match self.socket.try_write(&self.out_buffer) {
                        Ok(n) => {
                            trace!(length = n, "Outgoing data");
                            self.out_buffer.advance(n);
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            if self.handler_state.connection_state() == ConnectionState::Play {
                                error!("Unexpected error ocurred on write: {}", e);
                            }
                            self.handler_state.set_connection_state(ConnectionState::Disconnected);
                            break;
                        }
                    }
                    if self.out_buffer.is_empty() && self.handler_state.connection_state() == ConnectionState::Disconnected {
                        break;
                    }
                }
                Some(task) = self.connection_sync.1.recv() => {
                    let span = trace_span!("connection_task", state = %self.handler_state);
                    let _enter = span.enter();
                    task(&mut self);
                }
            }
        }
        if self.handler_state.connection_state() == ConnectionState::Disconnected {
            if let Some(uuid) = self.handler_state.player_uuid() {
                if self.server_tx.send(Box::new(move |server| server.player_leave(uuid))).is_err() {
                    warn!(%uuid, "Could not make server lose player, keep alive should clean up!");
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

    #[tracing::instrument(name = "read_packet", skip(self, preceding))]
    fn handle_packet_buffer(&mut self, preceding: usize, len: usize) -> Result<()> {
        let mut packet = self
            .in_buffer
            .split_to(preceding + len)
            .split_off(preceding)
            .freeze();
        if falcon_protocol::manager::PROTOCOL_MANAGER
            .process_packet(packet.read_var_i32()?, &mut packet, self)?
            .is_none()
        {
            if self.handler_state.connection_state() == Login
                || self.handler_state.connection_state() == Status
            {
                self.disconnect(String::from("{\"text\":\"Unsupported version!\"}"));
            }
            trace!("Unknown packet received, skipping!");
        }
        Ok(())
    }

    /// Reads a whole packet from the buffer and returns
    /// both the packet and the byte count with how many bytes it took to read the packet.
    ///
    /// (TODO: add compression and encryption mode!)
    ///
    /// Returns (preceding byte count, frame length)
    #[tracing::instrument(skip_all)]
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
                trace!(length = frame_length);
                return if buf.ensure_bytes_available(frame_length).is_ok() {
                    Ok(Some((i + 1, frame_length)))
                } else {
                    Ok(None)
                };
            }
        }
        Err(ErrorKind::InvalidPacketLength.into())
    }
}

impl MinecraftConnection for ClientConnection {
    fn get_address(&self) -> &SocketAddr {
        &self.addr
    }

    fn get_handler_state(&self) -> &PacketHandlerState {
        &self.handler_state
    }

    fn get_handler_state_mut(&mut self) -> &mut PacketHandlerState {
        &mut self.handler_state
    }

    fn get_server_link_mut(&mut self) -> &mut UnboundedSender<Box<McTask>> {
        &mut self.server_tx
    }

    fn get_connection_link(&self) -> UnboundedSender<Box<ConnectionTask>> {
        self.connection_sync.0.clone()
    }

    fn send_packet(&mut self, packet_id: i32, packet_out: &dyn PacketEncode) {
        if self.handler_state.connection_state() == ConnectionState::Disconnected {
            return;
        }
        let old_len = self.out_buffer.len();
        self.out_buffer.write_var_i32(packet_id);
        packet_out.to_buf(&mut self.out_buffer);
        let temp_buf = self.out_buffer.split_off(old_len);
        self.out_buffer.write_var_i32(temp_buf.len() as i32);
        self.out_buffer.unsplit(temp_buf);
    }

    fn reset_keep_alive(&mut self) {
        self.time_out.reset();
    }

    fn disconnect(&mut self, reason: String) { // TODO: change into ChatComponent
        let packet = falcon_protocol::build_disconnect_packet(reason);
        self.send_packet(0x00, &packet);
        self.handler_state.set_connection_state(ConnectionState::Disconnected);
    }
}
