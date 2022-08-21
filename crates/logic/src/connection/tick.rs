use crate::connection::ConnectionTask;
use crate::FalconConnection;
use falcon_core::network::connection::ConnectionLogic;
use falcon_core::network::ConnectionState;
use futures::StreamExt;
use mc_chat::{ChatColor, ChatComponent, ComponentStyle};
use tokio::net::TcpStream;
use tokio_util::codec::FramedRead;
use tracing::{trace_span, debug_span, trace};

use super::ConnectionReceiver;
use super::codec::{TcpWrite, FalconCodec};

impl FalconConnection {
    #[tracing::instrument(name = "client", skip_all, fields(address = %self.address()))]
    pub async fn start<R: ConnectionReceiver>(mut self, mut socket: TcpStream, mut receiver: R) {
        let (socket_read, mut socket_write) = socket.split();
        let mut socket_read = FramedRead::with_capacity(socket_read, FalconCodec, 4 * 1024);

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

                packet = socket_read.next() => {
                    let span = debug_span!("incoming_data", state = %self.state);
                    let _enter = span.enter();
                    if packet.is_none() {
                        self.state.set_connection_state(ConnectionState::Disconnected);
                        break;
                    }
                    let packet = packet.unwrap();
                    if let Err(error) = packet.and_then(|(packet_id, mut packet)| {
                        let span = trace_span!("packet", packet_id = %format!("{:#04X}", packet_id));
                        let _enter = span.enter();
                        if receiver.receive(packet_id, &mut packet, &mut self)?.is_none() {
                            let state = self.state.connection_state();
                            if state == ConnectionState::Login || state == ConnectionState::Status {
                                let style = ComponentStyle::with_version(self.state.protocol_id().unsigned_abs()).color_if_absent(ChatColor::Red);
                                self.disconnect(ChatComponent::from_text("Unsupported version!", style));
                            }
                            trace!("Unknown packet received, skipping!");
                        }
                        Ok(())
                    }) {
                        self.disconnect(ChatComponent::from_text(format!("Error on read: {}", error), ComponentStyle::with_version(self.state.protocol_id().unsigned_abs())));
                    }
                }

                result = TcpWrite::new(&mut socket_write, &mut self.write_buffer), if !self.write_buffer.is_empty() => {
                    if let Err(_) = result {
                        self.state.set_connection_state(ConnectionState::Disconnected);
                        break;
                    } else if self.write_buffer.is_empty() && self.state.connection_state() == ConnectionState::Disconnected {
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

