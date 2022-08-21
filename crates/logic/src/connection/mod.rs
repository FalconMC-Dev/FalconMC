use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use falcon_core::error::FalconCoreError;
use falcon_core::network::buffer::PacketBufferWrite;
use falcon_core::network::packet::PacketEncode;
use falcon_core::ShutdownHandle;
use mc_chat::ChatComponent;

use falcon_core::network::connection::ConnectionLogic;
use falcon_core::network::{ConnectionState, PacketHandlerState, UNKNOWN_PROTOCOL};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::time::{interval, Interval, MissedTickBehavior};
use tracing::{instrument, trace};

use crate::server::ServerWrapper;

pub use wrapper::ConnectionWrapper;

mod codec;
mod tick;
mod wrapper;

pub type SyncConnectionTask = dyn FnOnce(&mut FalconConnection) + Send + Sync;
pub type AsyncConnectionTask = dyn (FnOnce(&mut FalconConnection) -> Pin<Box<dyn Future<Output=()> + Send>>) + Send + Sync;

pub enum ConnectionTask {
    Sync(Box<SyncConnectionTask>),
    Async(Box<AsyncConnectionTask>),
}

pub trait ConnectionReceiver {
    fn receive(
        &mut self,
        packet_id: i32,
        bytes: &mut Bytes,
        connection: &mut FalconConnection,
    ) -> Result<Option<()>, FalconCoreError>;
}

#[derive(Debug)]
pub struct FalconConnection {
    shutdown: ShutdownHandle,
    server: ServerWrapper,
    task_rx: UnboundedReceiver<ConnectionTask>,
    wrapper: ConnectionWrapper,
    timeout: Interval,
    addr: SocketAddr,
    write_buffer: BytesMut,
    state: PacketHandlerState,
}

impl FalconConnection {
    pub async fn new(
        shutdown: ShutdownHandle,
        addr: SocketAddr,
        server: ServerWrapper,
    ) -> Self {
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
            write_buffer: BytesMut::with_capacity(4096),
            state: PacketHandlerState::new(UNKNOWN_PROTOCOL),
        }
    }

    pub fn reset_keep_alive(&mut self) {
        self.timeout.reset();
    }

    pub fn server(&self) -> &ServerWrapper {
        &self.server
    }

    pub fn wrapper(&self) -> ConnectionWrapper {
        self.wrapper.clone()
    }
}

impl ConnectionLogic for FalconConnection {
    fn address(&self) -> &std::net::SocketAddr {
        &self.addr
    }

    fn handler_state(&self) -> &falcon_core::network::PacketHandlerState {
        &self.state
    }

    fn handler_state_mut(&mut self) -> &mut falcon_core::network::PacketHandlerState {
        &mut self.state
    }

    fn send_packet<P: falcon_core::network::packet::PacketEncode>(
        &mut self,
        packet_id: i32,
        data: P,
    ) {
        self.send(PacketIdAndData::new(packet_id, data));
    }

    #[instrument(level = "trace", skip_all)]
    fn send<P: falcon_core::network::packet::PacketEncode>(&mut self, data: P) {
        if self.state.connection_state() == ConnectionState::Disconnected {
            return;
        }
        let old_len = self.write_buffer.len();
        data.to_buf(&mut self.write_buffer);
        let temp_buf = self.write_buffer.split_off(old_len);
        self.write_buffer.write_var_i32(temp_buf.len() as i32);
        trace!("{} bytes sent", temp_buf.len());
        self.write_buffer.unsplit(temp_buf);
    }

    #[instrument(level = "trace", skip_all)]
    fn disconnect(&mut self, reason: ChatComponent) {
        match self.state.connection_state() {
            ConnectionState::Play => falcon_send::send_play_disconnect(reason, self),
            _ => falcon_send::send_login_disconnect(reason, self),
        }
        self.state.set_connection_state(ConnectionState::Disconnected);
        trace!("Player connection marked as disconnected");
    }
}

struct PacketIdAndData<P: PacketEncode> {
    packet_id: i32,
    data: P,
}

impl<P: PacketEncode> PacketIdAndData<P> {
    pub fn new(packet_id: i32, data: P) -> Self {
        Self { packet_id, data }
    }
}

impl<P: PacketEncode> PacketEncode for PacketIdAndData<P> {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        buf.write_var_i32(self.packet_id);
        self.data.to_buf(buf);
    }
}
