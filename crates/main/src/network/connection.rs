use std::io::Cursor;
use std::net::SocketAddr;
use std::time::Duration;

use bytes::{Buf, BytesMut};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::{interval, Interval, MissedTickBehavior};

use falcon_core::network::buffer::{ByteLimitCheck, PacketBufferRead, PacketBufferWrite};
use falcon_core::network::connection::{ClientConnection, ConnectionActor, ConnectionData, ConnectionTask, ConnectionWrapper};
use falcon_core::network::packet::PacketEncode;
use falcon_core::network::ConnectionState::{Login, Status};
use falcon_core::network::{ConnectionState, PacketHandlerState};
use falcon_core::server::{McTask, ServerWrapper};
use falcon_core::ShutdownHandle;
use falcon_core::network::UNKNOWN_PROTOCOL;

use anyhow::Result;
use mc_chat::{ChatColor, ChatComponent, ComponentStyle};

pub(crate) async fn new_connection(
    shutdown_handle: ShutdownHandle,
    socket: TcpStream,
    addr: SocketAddr,
    server_tx: UnboundedSender<Box<McTask>>,
) -> ClientConnection {
    let mut time_out = interval(Duration::from_secs(30));
    time_out.set_missed_tick_behavior(MissedTickBehavior::Delay);
    time_out.tick().await;
    ClientConnection::new(
        shutdown_handle,
        socket,
        addr,
        PacketHandlerState::new(UNKNOWN_PROTOCOL),
        time_out,
        ServerWrapper::new(server_tx),
    )
}

#[tracing::instrument(name = "client", skip_all, fields(address = %self.addr))]
pub(crate) async fn connection_loop(mut connection: ClientConnection) {
    loop {
        tokio::select! {
                _ = self.shutdown_handle.wait_for_shutdown() => {
                    break;
                }
                _ = self.time_out.tick() => {
                    self.disconnect(ChatComponent::from_text(
                        "Did not receive Keep alive packet!",
                        ComponentStyle::with_version(self.handler_state.protocol_id().unsigned_abs())
                    ));
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
                                self.disconnect(ChatComponent::from_text(
                                    "Error while reading packet",
                                    ComponentStyle::with_version(self.handler_state.protocol_id().unsigned_abs())
                                ));
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            if self.handler_state.connection_state() == ConnectionState::Play {
                                error!("Unexpected error ocurred on read: {}", e);
                            }
                            self.disconnect(ChatComponent::from_text(
                                format!("Unexpected error ocurred on read: {}", e),
                                ComponentStyle::with_version(self.handler_state.protocol_id().unsigned_abs())
                            ));
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
    if connection.handler_state().connection_state() == ConnectionState::Disconnected {
        if let Some(uuid) = connection.handler_state().player_uuid() {
            if connection.server().execute(move |server| server.player_leave(uuid)).is_err() {
                warn!(%uuid, "Could not make server lose player, keep alive should clean up!");
            }
        }
    }
}

fn read_packets(connection: &mut ClientConnection) -> Result<()> {
    while let Some((preceding, len)) = parse_frame(connection)? {
        handle_packet_buffer(connection, preceding, len)?;
    }
    Ok(())
}

#[tracing::instrument(name = "read_packet", skip(connection, preceding))]
fn handle_packet_buffer(connection: &mut ClientConnection, preceding: usize, len: usize) -> Result<()> {
    let mut packet = connection
        .in_buffer
        .split_to(preceding + len)
        .split_off(preceding)
        .freeze();
    if falcon_plugins::manager::PROTOCOL_MANAGER
        .process_packet(packet.read_var_i32()?, &mut packet, connection)?
        .is_none()
    {
        if connection.handler_state().connection_state() == Login
            || connection.handler_state().connection_state() == Status
        {
            connection.disconnect(ChatComponent::from_text(
                "Unsupported version!",
                ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs()).color_if_absent(ChatColor::Red)
            ));
        }
        trace!("Unknown packet received, skipping!");
    }
    Ok(())
}

/// Reads a whole packet from the buffer and returns
/// both the packet and the byte count with how many bytes it took to read the packet.
///
/// Returns (preceding byte count, frame length)
#[tracing::instrument(skip_all)]
fn parse_frame(connection: &ClientConnection) -> Result<Option<(usize, usize)>> {
    let mut buf = Cursor::new(&connection.in_buffer[..]);
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
    Err(anyhow!("The packet length was longer than 21 bits!"))
}
