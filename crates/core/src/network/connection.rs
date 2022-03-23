use std::net::SocketAddr;
use ignore_result::Ignore;
use tokio::sync::mpsc::UnboundedSender;

use crate::network::packet::PacketEncode;
use crate::network::PacketHandlerState;
use crate::server::ServerWrapper;

pub type ConnectionTask = dyn FnOnce(&mut dyn MinecraftConnection) + Send + Sync;

pub trait ConnectionData {
    fn address(&self) -> &SocketAddr;

    fn handler_state(&self) -> &PacketHandlerState;

    fn handler_state_mut(&mut self) -> &mut PacketHandlerState;

    fn server(&mut self) -> &mut ServerWrapper;

    fn wrapper(&self) -> ConnectionWrapper;

    fn send_packet(&mut self, packet_id: i32, packet_out: &dyn PacketEncode);
}

pub trait ConnectionActor {
    fn reset_keep_alive(&mut self);

    fn disconnect(&mut self, reason: String); // TODO: change into ChatComponent
}

pub trait MinecraftConnection: ConnectionData + ConnectionActor {}

impl<T: ConnectionData + ConnectionActor> MinecraftConnection for T {}

#[derive(Debug)]
pub struct ConnectionWrapper {
    link: UnboundedSender<Box<ConnectionTask>>,
}

impl ConnectionActor for ConnectionWrapper {
    fn reset_keep_alive(&mut self) {
        self.link.send(Box::new(|conn| {
            conn.reset_keep_alive();
        })).ignore();
    }

    fn disconnect(&mut self, reason: String) {
        self.link.send(Box::new(move |conn| {
            conn.disconnect(reason);
        })).ignore();
    }
}

impl ConnectionWrapper {
    pub fn new(link: UnboundedSender<Box<ConnectionTask>>) -> Self {
        ConnectionWrapper {
            link,
        }
    }

    pub fn send_packet<P>(&self, packet_id: i32, packet: P)
    where
        P: PacketEncode + Send + Sync + 'static,
    {
        self.execute(move |connection| {
            let packet_out = packet;
            connection.send_packet(packet_id, &packet_out);
        })
    }

    pub fn execute<T>(&self, task: T)
        where
            T: FnOnce(&mut dyn MinecraftConnection) + Send + Sync + 'static,
    {
        self.link.send(Box::new(task)).ignore();
    }
}
