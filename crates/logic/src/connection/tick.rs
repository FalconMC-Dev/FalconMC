use crate::connection::ConnectionTask;
use crate::FalconConnection;
use bytes::{Buf, Bytes};
use falcon_core::{error::FalconCoreError, network::ConnectionState};
use falcon_packet_core::{PacketRead, ReadError, VarI32};
use mc_chat::{ChatColor, ChatComponent, ComponentStyle};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{debug_span, trace, trace_span};

use super::{reader::SocketRead, ConnectionReceiver};

impl FalconConnection {
    #[tracing::instrument(name = "client", skip_all, fields(address = %self.address()))]
    pub async fn start<R: ConnectionReceiver>(mut self, mut socket: TcpStream, mut receiver: R) {
        let (mut socket_readhalf, mut socket_writehalf) = socket.split();
        let mut socket_read = SocketRead::new(-1);

        loop {
            tokio::select! {
                _ = self.shutdown.wait_for_shutdown() => {
                    break;
                }

                _ = self.timeout.tick() => {
                    let style = ComponentStyle::with_version(self.handler_state().protocol_id().unsigned_abs());
                    self.disconnect(ChatComponent::from_text(
                        "Did not receive Keep alive packet!",
                        style
                    ));
                }

                task = self.task_rx.recv() => {
                    let task = match task {
                        Some(task) => task,
                        None => continue,
                    };
                    let span = debug_span!("connection_task", state = %self.state);
                    let _enter = span.enter();
                    match task {
                        ConnectionTask::Sync(task) => {
                            task(&mut self)
                        }
                        ConnectionTask::Async(task) => {
                            task(&mut self).await
                        }
                    }
                }

                n = socket_readhalf.read_buf(&mut socket_read) => {
                    let span = debug_span!("incoming_data", state = %self.state);
                    let _enter = span.enter();
                    match n {
                        Ok(n) => {
                            if n == 0 {
                                self.state.set_connection_state(ConnectionState::Disconnected);
                                break;
                            }
                            while let Some(packet) = socket_read.next_packet() {
                                if let Err(error) = process_packet(&mut self, packet, &mut receiver) {
                                    self.disconnect(ChatComponent::from_text(format!("Error on read: {}", error), ComponentStyle::with_version(self.state.protocol_id().unsigned_abs())));
                                }
                            }
                        }
                        Err(error) => {
                            self.disconnect(ChatComponent::from_text(format!("Error on read: {}", error), ComponentStyle::with_version(self.state.protocol_id().unsigned_abs())));
                        }
                    }
                }

                res = socket_writehalf.write_all_buf(&mut self.write_buffer), if self.write_buffer.has_remaining() => {
                    if res.is_err() {
                        self.state.set_connection_state(ConnectionState::Disconnected);
                        break;
                    } else if !self.write_buffer.has_remaining() && self.state.connection_state() == ConnectionState::Disconnected {
                        break;
                    }
                }
            }
        }
        if self.handler_state().connection_state() == ConnectionState::Disconnected {
            if let Some(uuid) = self.handler_state().player_uuid() {
                self.server().player_leave(uuid);
            }
        }
    }
}

fn process_packet<R: ConnectionReceiver>(
    connection: &mut FalconConnection,
    mut packet: Bytes,
    receiver: &mut R,
) -> Result<(), ReceiveError> {
    let packet_id = VarI32::read(&mut packet)?.val();
    let span = trace_span!("packet", packet_id = %format!("{:#04X}", packet_id));
    let _enter = span.enter();
    if !receiver.receive(packet_id, &mut packet, connection)? {
        let state = connection.handler_state().connection_state();
        if state == ConnectionState::Login || state == ConnectionState::Status {
            let style = ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs()).color_if_absent(ChatColor::Red);
            connection.disconnect(ChatComponent::from_text("Unsupported version!", style));
        }
        trace!("Unknown packet received, skipping!");
    }
    Ok(())
}

#[derive(Error, Debug)]
enum ReceiveError {
    #[error("Core error")]
    Core(#[from] FalconCoreError),
    #[error("packet read error")]
    Read(#[from] ReadError),
}
