use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use falcon_core::network::{ConnectionState, PacketHandlerState, UNKNOWN_PROTOCOL};
use falcon_core::ShutdownHandle;
use falcon_packet_core::{ReadError, WriteError};
use mc_chat::ChatComponent;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::time::{interval, Interval, MissedTickBehavior};
use tracing::{instrument, trace};
pub use wrapper::ConnectionWrapper;

use self::writer::SocketWrite;
use crate::server::ServerWrapper;

// mod codec;
pub mod handler;
pub mod reader;
mod tick;
mod wrapper;
pub mod writer;

pub type SyncConnectionTask = dyn FnOnce(&mut FalconConnection) -> Result<()> + Send + Sync ;
pub type AsyncConnectionTask = dyn (FnOnce(&mut FalconConnection) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>) + Send + Sync;

pub enum ConnectionTask {
    Sync(Box<SyncConnectionTask>),
    Async(Box<AsyncConnectionTask>),
}

pub trait ConnectionReceiver {
    fn receive(&mut self, packet_id: i32, bytes: &mut Bytes, connection: &mut FalconConnection) -> Result<bool, ReadError>;
}

#[derive(Debug)]
pub struct FalconConnection {
    shutdown: ShutdownHandle,
    server: ServerWrapper,
    task_rx: UnboundedReceiver<ConnectionTask>,
    wrapper: ConnectionWrapper,
    timeout: Interval,
    addr: SocketAddr,
    write_buffer: SocketWrite,
    state: PacketHandlerState,
}

impl FalconConnection {
    pub async fn new(shutdown: ShutdownHandle, addr: SocketAddr, server: ServerWrapper) -> Self {
        let mut timeout = interval(Duration::from_secs(30));
        timeout.set_missed_tick_behavior(MissedTickBehavior::Delay);
        timeout.tick().await;
        let (sender, receiver) = unbounded_channel();
        Self {
            shutdown,
            server,
            wrapper: ConnectionWrapper::new(sender),
            task_rx: receiver,
            timeout,
            addr,
            write_buffer: SocketWrite::new(-1),
            state: PacketHandlerState::new(UNKNOWN_PROTOCOL),
        }
    }

    pub fn reset_keep_alive(&mut self) { self.timeout.reset(); }

    pub fn server(&self) -> &ServerWrapper { &self.server }

    pub fn wrapper(&self) -> ConnectionWrapper { self.wrapper.clone() }
}

impl FalconConnection {
    pub fn address(&self) -> &std::net::SocketAddr { &self.addr }

    pub fn handler_state(&self) -> &falcon_core::network::PacketHandlerState { &self.state }

    pub fn handler_state_mut(&mut self) -> &mut falcon_core::network::PacketHandlerState { &mut self.state }

    #[instrument(level = "trace", skip_all)]
    pub fn send<F>(&mut self, write_fn: F) -> Result<(), WriteError>
    where
        F: FnOnce(&mut SocketWrite, i32) -> Result<(), WriteError>,
    {
        if self.state.connection_state() == ConnectionState::Disconnected {
            return Ok(());
        }
        write_fn(&mut self.write_buffer, self.state.protocol_id())?;
        self.write_buffer.finish();
        Ok(())
    }

    pub fn send_packet<T, F>(&mut self, packet: T, write_fn: F) -> Result<(), WriteError>
    where
        F: FnOnce(T, &mut SocketWrite, i32) -> Result<bool, WriteError>,
    {
        self.send(move |buffer, protocol| {
            if !write_fn(packet, buffer, protocol)? {
                // trace!("Unresolved packet");
            }
            Ok(())
        })
    }

    #[instrument(level = "trace", skip_all)]
    pub fn disconnect(&mut self, reason: ChatComponent) {
        match self.state.connection_state() {
            ConnectionState::Play => self.send_packet(reason, falcon_send::write_play_disconnect).ok(),
            _ => self.send_packet(reason, falcon_send::write_login_disconnect).ok(),
        };
        self.state.set_connection_state(ConnectionState::Disconnected);
        trace!("Player connection marked as disconnected");
    }
}
