use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use falcon_core::network::{ConnectionState, PacketHandlerState, UNKNOWN_PROTOCOL};
use falcon_core::ShutdownHandle;
use falcon_packet_core::WriteError;
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

pub trait SyncConnectionTask: Send + Sync {
    fn run(self: Box<Self>, connection: &mut FalconConnection) -> Result<()>;
}

pub trait SyncFutConnectionTask: Send + Sync {
    fn run(self: Box<Self>, connection: &mut FalconConnection) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

pub enum ConnectionTask {
    Sync(Box<dyn SyncConnectionTask>),
    Async(Box<dyn SyncFutConnectionTask>),
}

pub trait ConnectionReceiver {
    fn receive(&mut self, packet_id: i32, bytes: &mut Bytes, connection: &mut FalconConnection) -> Result<bool>;
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

    pub fn state(&self) -> &falcon_core::network::PacketHandlerState { &self.state }

    pub fn state_mut(&mut self) -> &mut falcon_core::network::PacketHandlerState { &mut self.state }

    #[instrument(level = "trace", skip_all)]
    pub fn send<F>(&mut self, write_fn: F) -> Result<(), WriteError>
    where
        F: FnOnce(&mut SocketWrite, i32) -> Result<(), WriteError>,
    {
        if self.state.connection_state == ConnectionState::Disconnected {
            return Ok(());
        }
        write_fn(&mut self.write_buffer, self.state.protocol_id)?;
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
        match self.state.connection_state {
            ConnectionState::Play => self.send_packet(reason, falcon_send::write_play_disconnect).ok(),
            _ => self.send_packet(reason, falcon_send::write_login_disconnect).ok(),
        };
        self.state.connection_state = ConnectionState::Disconnected;
        trace!("Player connection marked as disconnected");
    }
}

impl<F, E> SyncConnectionTask for F
where
    E: Error + Send + Sync + 'static,
    F: FnOnce(&mut FalconConnection) -> Result<(), E> + Send + Sync,
{
    fn run(self: Box<Self>, server: &mut FalconConnection) -> Result<()> { Ok(self(server)?) }
}

impl<F, E> SyncFutConnectionTask for F
where
    E: Error + Send + Sync + 'static,
    F: FnOnce(&mut FalconConnection) -> Pin<Box<dyn Future<Output = Result<(), E>> + Send>> + Send + Sync + 'static,
{
    fn run(self: Box<F>, server: &mut FalconConnection) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> { Box::pin(async { Ok(self(server).await?) }) }
}
