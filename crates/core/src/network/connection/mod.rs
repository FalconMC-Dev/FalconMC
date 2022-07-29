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

use falcon_core::network::PacketHandlerState;
use falcon_core::ShutdownHandle;
pub use wrapper::ConnectionWrapper;
use crate::network::buffer::PacketBufferWrite;

mod wrapper;

pub type SyncConnectionTask<D> = dyn FnOnce(&mut D) + Send + Sync;
pub type AsyncConnectionTask<D> = dyn (FnOnce(&mut D) -> Pin<Box<dyn Future<Output=()>>>) + Send + Sync;

pub enum ConnectionTask<D: ConnectionDriver<L>, L: ConnectionLogic> {
    Sync(Box<SyncConnectionTask<D>>),
    Async(Box<AsyncConnectionTask<D>>),
    _Prohibited(Infallible, PhantomData<L>),
}

#[async_trait::async_trait]
pub trait ConnectionLogic: Debug {
    async fn disconnect(&self, reason: ChatComponent);
}

#[async_trait::async_trait]
pub trait ConnectionDriver<L: ConnectionLogic>: Debug {
    type Error: Display + From<Error>;

    fn logic(&self) -> &L;

    fn logic_mut(&mut self) -> &mut L;

    fn addr(&self) -> &SocketAddr;

    fn handler_state(&self) -> &PacketHandlerState;

    fn handler_state_mut(&mut self) -> &mut PacketHandlerState;

    async fn receive(&mut self) -> Result<(), Self::Error>;

    async fn send<P: PacketBufferWrite + ?Sized>(&mut self, packet_out: P) -> Result<(), Self::Error>;

    fn on_disconnect(&mut self);
}

#[derive(Debug)]
pub struct MinecraftConnection<D: ConnectionDriver<L>, L: ConnectionLogic> {
    shutdown: ShutdownHandle,
    driver: D,
    task_rx: UnboundedReceiver<ConnectionTask<D, L>>,
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> Deref for MinecraftConnection<D, L> {
    type Target = L;

    fn deref(&self) -> &Self::Target {
        self.driver.logic()
    }
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> DerefMut for MinecraftConnection<D, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.driver.logic_mut()
    }
}

impl<D: ConnectionDriver<L>, L: ConnectionLogic> MinecraftConnection<D, L> {
    pub fn new(shutdown: ShutdownHandle, driver: D, task_rx: UnboundedReceiver<ConnectionTask<D, L>>) -> Self {
        Self {
            shutdown,
            driver,
            task_rx,
        }
    }

    pub fn shutdown_handle(&mut self) -> &mut ShutdownHandle {
        &mut self.shutdown
    }

    pub fn driver(&self) -> &D {
        &self.driver
    }

    pub fn driver_mut(&mut self) -> &mut D {
        &mut self.driver
    }

    pub async fn start(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown.wait_for_shutdown() => {
                    break;
                }
                res = self.driver.receive() => {
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
                    let span = trace_span!("connection_task", state = %self.driver.handler_state());
                    let _enter = span.enter();
                    match task {
                        ConnectionTask::Sync(task) => {
                            task(&mut self.driver)
                        }
                        ConnectionTask::Async(task) => {
                            task(&mut self.driver).await
                        }
                        ConnectionTask::_Prohibited(..) => {
                            unreachable!("This enum variant of `ConnectionTask` should never be used");
                        }
                    }
                }
            }
        }
        self.driver.on_disconnect();
        /*if self.driver.handler_state().connection_state() == ConnectionState::Disconnected {
            if let Some(uuid) = self.driver.handler_state().player_uuid() {
                // self.server().player_leave(uuid);
            }
        }*/
    }
}