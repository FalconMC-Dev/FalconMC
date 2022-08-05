use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::io::Error;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use mc_chat::ChatComponent;

use tokio::sync::mpsc::UnboundedReceiver;

use falcon_core::ShutdownHandle;
use falcon_core::network::PacketHandlerState;
use falcon_core::network::packet::PacketEncode;

pub use wrapper::ConnectionWrapper;

mod wrapper;

pub type SyncConnectionTask<L> = dyn FnOnce(&mut L) + Send + Sync;
pub type AsyncConnectionTask<L> = dyn (FnOnce(&mut L) -> Pin<Box<dyn Future<Output=()>>>) + Send + Sync;

pub enum ConnectionTask<D: ConnectionDriver, L: ConnectionLogic<D>> {
    Sync(Box<SyncConnectionTask<L>>),
    Async(Box<AsyncConnectionTask<L>>),
    _Prohibited(Infallible, PhantomData<D>),
}

pub trait ConnectionLogic<D: ConnectionDriver>: Debug {
    fn driver(&self) -> &D;

    fn driver_mut(&mut self) -> &mut D;

    fn disconnect(&mut self, reason: ChatComponent);
}

#[async_trait::async_trait]
pub trait ConnectionDriver: Debug + Send + Sync {
    type Error: Display + From<Error>;

    fn addr(&self) -> &SocketAddr;

    fn handler_state(&self) -> &PacketHandlerState;

    fn handler_state_mut(&mut self) -> &mut PacketHandlerState;

    fn reset_timeout(&mut self);

    async fn receive(&mut self) -> Result<(), Self::Error>;

    fn send_packet<P: PacketEncode + ?Sized>(&mut self, packet_id: i32, data: P);

    fn send<P: PacketEncode + ?Sized>(&mut self, data: P);

    fn on_disconnect(&mut self);
}

#[derive(Debug)]
pub struct MinecraftConnection<D: ConnectionDriver, L: ConnectionLogic<D>> {
    shutdown: ShutdownHandle,
    logic: L,
    task_rx: UnboundedReceiver<ConnectionTask<D, L>>,
}

impl<D: ConnectionDriver, L: ConnectionLogic<D>> Deref for MinecraftConnection<D, L> {
    type Target = L;

    fn deref(&self) -> &Self::Target {
        &self.logic
    }
}

impl<D: ConnectionDriver, L: ConnectionLogic<D>> DerefMut for MinecraftConnection<D, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.logic
    }
}

impl<D: ConnectionDriver, L: ConnectionLogic<D>> MinecraftConnection<D, L> {
    pub fn new(shutdown: ShutdownHandle, logic: L, task_rx: UnboundedReceiver<ConnectionTask<D, L>>) -> Self {
        Self {
            shutdown,
            logic,
            task_rx,
        }
    }

    pub fn shutdown_handle(&mut self) -> &mut ShutdownHandle {
        &mut self.shutdown
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
                res = self.logic.driver_mut().receive() => {
                    if let Err(error) = res {
                        // do something error
                    }
                }
                /*_ = self.driver.timeout().tick() => {
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
                }*/
                task = self.task_rx.recv() => {
                    let task = match task {
                        Some(task) => task,
                        None => continue,
                    };
                    let span = trace_span!("connection_task", state = %self.logic.driver().handler_state());
                    let _enter = span.enter();
                    match task {
                        ConnectionTask::Sync(task) => {
                            task(&mut self.logic)
                        }
                        ConnectionTask::Async(task) => {
                            task(&mut self.logic).await
                        }
                        ConnectionTask::_Prohibited(..) => {
                            unreachable!("This enum variant of `ConnectionTask` should never be used");
                        }
                    }
                }
            }
        }
        self.logic.driver_mut().on_disconnect();
        /*if self.driver.handler_state().connection_state() == ConnectionState::Disconnected {
            if let Some(uuid) = self.driver.handler_state().player_uuid() {
                // self.server().player_leave(uuid);
            }
        }*/
    }
}
