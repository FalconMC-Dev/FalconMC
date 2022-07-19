use std::fmt::Debug;
use std::future::Future;
use std::io::Error;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use futures::StreamExt;
use mc_chat::ChatComponent;

use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Interval;
use tokio_util::codec::{Decoder, Encoder, Framed};

use falcon_core::network::{ConnectionState, PacketHandlerState};
use falcon_core::server::ServerWrapper;
use falcon_core::ShutdownHandle;
pub use wrapper::ConnectionWrapper;
use crate::network::buffer::PacketBufferWrite;

mod wrapper;

pub type SyncConnectionTask<D, L> = dyn FnOnce(&mut ClientConnection<D, L>) + Send + Sync;
pub type AsyncConnectionTask<D, L> = dyn (FnOnce(&mut ClientConnection<D, L>) -> Pin<Box<dyn Future<Output=()>>>) + Send + Sync;

pub enum ConnectionTask<D: ConnectionDriver<L>, L: ConnectionLogic> {
    Sync(Box<SyncConnectionTask<D, L>>),
    Async(Box<AsyncConnectionTask<D, L>>),
}

#[async_trait::async_trait]
pub trait ConnectionLogic: Debug {
    async fn disconnect(&self, reason: ChatComponent);
}

#[async_trait::async_trait]
pub trait ConnectionDriver<L: ConnectionLogic>: Debug {
    type PacketIn;
    type PacketOut;
    type Error: From<Error>;
    type PacketCodec: Decoder<Item=Self::PacketIn, Error=Self::Error> + Encoder<Self::PacketOut, Error=Self::Error>;

    fn addr(&self) -> &SocketAddr;

    fn handler_state(&self) -> &PacketHandlerState;

    fn handler_state_mut(&self) -> &mut PacketHandlerState;

    fn network_io(&mut self) -> &mut Framed<TcpStream, Self::PacketCodec>;

    async fn on_receive(&mut self, packet_in: Self::PacketIn) -> Result<(), Self::Error>;

    async fn send<P: PacketBufferWrite + ?Sized>(&mut self, packet_out: P) -> Result<(), Self::Error>;

    fn server(&mut self) -> &mut ServerWrapper<Self, L> where Self: Sized, Self: Debug;
}

#[derive(Debug)]
pub struct ClientConnection<D: ConnectionDriver<L>, L: ConnectionLogic> {
    shutdown: ShutdownHandle,
    driver: D,
    logic: L,
    timeout: Interval,
    task_rx: UnboundedReceiver<ConnectionTask<D, L>>,
    wrapper: ConnectionWrapper<D, L>,
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> Deref for ClientConnection<D, L> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.driver
    }
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> DerefMut for ClientConnection<D, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.driver
    }
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> ClientConnection<D, L> {
    pub fn new(shutdown: ShutdownHandle, driver: D, logic: L, timeout: Interval) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            shutdown,
            driver,
            logic,
            timeout,
            task_rx: receiver,
            wrapper: ConnectionWrapper::new(sender),
        }
    }

    pub fn shutdown_handle(&mut self) -> &mut ShutdownHandle {
        &mut self.shutdown
    }

    pub fn wrapper(&self) -> ConnectionWrapper<D, L> {
        self.wrapper.clone()
    }

    pub fn logic(&self) -> &L {
        &self.logic
    }

    pub fn logic_mut(&mut self) -> &mut L {
        &mut self.logic
    }

    pub async fn start(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown.wait_for_shutdown() => {
                    break;
                }
                _ = self.timeout.tick() => {
                    /*let style = ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs());
                    disconnect(&mut connection, ChatComponent::from_text(
                        "Did not receive Keep alive packet!",
                        style
                    ));*/
                }
                packet = self.driver.network_io().next() => {
                    if let Some(packet) = packet {
                        match packet {
                            Ok(packet) => {
                                if let Err(error) = self.driver.on_receive(packet).await {
                                    /*let style = ComponentStyle::with_version(connection.handler_state().protocol_id().unsigned_abs());
                                    disconnect(&mut connection, ChatComponent::from_text(
                                        "Error while reading packet",
                                        style
                                    ));*/
                                }
                            }
                            Err(e) => {
                                // something error related
                            }
                        }
                    }
                }
                task = self.task_rx.recv() => {
                    let task = match task {
                        Some(task) => task,
                        None => continue,
                    };
                    let span = trace_span!("connection_task", state = %self.handler_state());
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
            }
        }
        if self.handler_state().connection_state() == ConnectionState::Disconnected {
            if let Some(uuid) = self.handler_state().player_uuid() {
                // self.server().player_leave(uuid);
            }
        }
    }
}