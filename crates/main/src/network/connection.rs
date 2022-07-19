use std::io::Cursor;
use std::net::SocketAddr;
use std::time::Duration;

use bytes::Buf;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{interval, MissedTickBehavior};

use falcon_core::network::buffer::{ByteLimitCheck, PacketBufferRead};
use falcon_core::network::connection::ClientConnection;
use falcon_core::network::{ConnectionState, PacketHandlerState};
use falcon_core::server::{McTask, ServerWrapper};
use falcon_core::ShutdownHandle;
use falcon_core::network::UNKNOWN_PROTOCOL;

use falcon_logic::connection::disconnect;

use anyhow::Result;
use mc_chat::{ChatColor, ChatComponent, ComponentStyle};
use falcon_logic::ServerLogic;
use falcon_receive::falcon_process_packet;

pub(crate) async fn new_connection(
    shutdown_handle: ShutdownHandle,
    socket: TcpStream,
    addr: SocketAddr,
    server_tx: UnboundedSender<Box<McTask>>,
) {
    let mut time_out = interval(Duration::from_secs(30));
    time_out.set_missed_tick_behavior(MissedTickBehavior::Delay);
    time_out.tick().await;
    let connection = ClientConnection::new(
        shutdown_handle,
        socket,
        addr,
        PacketHandlerState::new(UNKNOWN_PROTOCOL),
        time_out,
        ServerWrapper::new(server_tx),
    );

    connection_loop(connection).await
}

#[tracing::instrument(name = "client", skip_all, fields(address = %connection.address()))]
pub(crate) async fn connection_loop(mut connection: ClientConnection) {
    loop {
        tokio::select! {
            _ = connection.shutdown_handle.wait_for_shutdown() => {
                break;
            }
            _ = connection.time_out.tick() => {
                let style = ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs());
                disconnect(&mut connection, ChatComponent::from_text(
                    "Did not receive Keep alive packet!",
                    style
                ));
            }
            readable = connection.socket.readable(), if connection.handler_state().connection_state() != ConnectionState::Disconnected => {
                let span = trace_span!("incoming_data", state = %connection.handler_state());
                let _enter = span.enter();
                if let Err(e) = readable {
                    error!("Error on read: {}", e);
                    connection.handler_state_mut().set_connection_state(ConnectionState::Disconnected);
                    break;
                }
                match connection.socket.try_read_buf(&mut connection.in_buffer) {
                    Ok(0) => {
                        trace!("Connection lost!");
                        connection.handler_state_mut().set_connection_state(ConnectionState::Disconnected);
                        break;
                    }
                    Ok(n) => {
                        trace!(length = n, "Data received!");
                        if let Err(e) = read_packets(&mut connection) {
                            debug!("Read error: {}", e);
                            let style = ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs());
                            disconnect(&mut connection, ChatComponent::from_text(
                                "Error while reading packet",
                                style
                            ));
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        if connection.handler_state().connection_state() == ConnectionState::Play {
                            error!("Unexpected error ocurred on read: {}", e);
                        }
                        let style = ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs());
                        disconnect(&mut connection, ChatComponent::from_text(
                            format!("Unexpected error ocurred on read: {}", e),
                            style
                        ));
                    }
                }
            }
            writable = connection.socket.writable(), if !connection.out_buffer.is_empty() => {
                let span = trace_span!("outgoing_data", state = %connection.handler_state());
                let _enter = span.enter();
                if let Err(e) = writable {
                    error!("Error on write: {}", e);
                    connection.handler_state_mut().set_connection_state(ConnectionState::Disconnected);
                    break;
                }
                match connection.socket.try_write(&connection.out_buffer) {
                    Ok(n) => {
                        trace!(length = n, "Outgoing data");
                        connection.out_buffer.advance(n);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        if connection.handler_state().connection_state() == ConnectionState::Play {
                            error!("Unexpected error ocurred on write: {}", e);
                        }
                        connection.handler_state_mut().set_connection_state(ConnectionState::Disconnected);
                        break;
                    }
                }
                if connection.out_buffer.is_empty() && connection.handler_state().connection_state() == ConnectionState::Disconnected {
                    break;
                }
            }
            Some(task) = connection.connection_sync.1.recv() => {
                let span = trace_span!("connection_task", state = %connection.handler_state());
                let _enter = span.enter();
                task(&mut connection);
            }
        }
    }
    if connection.handler_state().connection_state() == ConnectionState::Disconnected {
        if let Some(uuid) = connection.handler_state().player_uuid() {
            connection.server().player_leave(uuid);
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
    let packet_id = packet.read_var_i32()?;
    let handler_state = connection.handler_state();
    let span = trace_span!("default", packet_id = %format!("{:#04X}", packet_id), state = ?handler_state.connection_state());
    let _enter = span.enter();
    if falcon_process_packet(packet_id, &mut packet, connection)?.is_none() {
        if connection.handler_state().connection_state() == ConnectionState::Login
            || connection.handler_state().connection_state() == ConnectionState::Status
        {
            let style = ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs()).color_if_absent(ChatColor::Red);
            disconnect(connection, ChatComponent::from_text(
                "Unsupported version!",
                style
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
