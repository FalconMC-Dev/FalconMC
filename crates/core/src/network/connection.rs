use std::net::SocketAddr;
use bytes::BytesMut;
use ignore_result::Ignore;
use mc_chat::ChatComponent;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::Interval;

use crate::network::packet::PacketEncode;
use crate::network::{ConnectionState, PacketHandlerState};
use crate::network::buffer::PacketBufferWrite;
use crate::server::ServerWrapper;
use crate::ShutdownHandle;

pub type ConnectionTask = dyn FnOnce(&mut ClientConnection) + Send + Sync;

pub struct ClientConnection {
    shutdown_handle: ShutdownHandle,
    // connection data
    pub socket: TcpStream,
    addr: SocketAddr,
    // packet handling
    handler_state: PacketHandlerState,
    pub out_buffer: BytesMut,
    pub in_buffer: BytesMut,
    // synchronization
    pub time_out: Interval,
    server_tx: ServerWrapper,
    pub connection_sync: (
        UnboundedSender<Box<ConnectionTask>>,
        UnboundedReceiver<Box<ConnectionTask>>,
    ),
}

impl ClientConnection {
    pub fn new(
        shutdown_handle: ShutdownHandle,
        socket: TcpStream,
        addr: SocketAddr,
        handler_state: PacketHandlerState,
        time_out: Interval,
        server_tx: ServerWrapper,
    ) -> Self {
        ClientConnection {
            shutdown_handle,
            socket,
            addr,
            handler_state,
            out_buffer: BytesMut::with_capacity(4096),
            in_buffer: BytesMut::with_capacity(4096),
            time_out,
            server_tx,
            connection_sync: tokio::sync::mpsc::unbounded_channel(),
        }
    }

    pub fn shutdown_handle(&mut self) -> &mut ShutdownHandle {
        &mut self.shutdown_handle
    }

    pub fn address(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn handler_state(&self) -> &PacketHandlerState {
        &self.handler_state
    }

    pub fn handler_state_mut(&mut self) -> &mut PacketHandlerState {
        &mut self.handler_state
    }

    pub fn server(&mut self) -> &mut ServerWrapper {
        &mut self.server_tx
    }

    pub fn wrapper(&self) -> ConnectionWrapper {
        ConnectionWrapper::new(self.connection_sync.0.clone())
    }

    pub fn send_packet(&mut self, packet_id: i32, packet_out: &dyn PacketEncode) {
        if self.handler_state.connection_state() == ConnectionState::Disconnected {
            return;
        }
        let old_len = self.out_buffer.len();
        self.out_buffer.write_var_i32(packet_id);
        packet_out.to_buf(&mut self.out_buffer);
        let temp_buf = self.out_buffer.split_off(old_len);
        self.out_buffer.write_var_i32(temp_buf.len() as i32);
        self.out_buffer.unsplit(temp_buf);
    }

    pub fn reset_keep_alive(&mut self) {
        self.time_out.reset();
    }

    pub fn disconnect(&mut self, reason: ChatComponent) {
        /*match self.handler_state.connection_state() {
            ConnectionState::Play => falcon_default_protocol::clientbound::send_play_disconnect(reason, self),
            _ => falcon_default_protocol::clientbound::send_login_disconnect(reason, self),
        }*/
        self.handler_state.set_connection_state(ConnectionState::Disconnected);
    }
}

#[derive(Debug)]
pub struct ConnectionWrapper {
    link: UnboundedSender<Box<ConnectionTask>>,
}

impl ConnectionWrapper {
    pub fn reset_keep_alive(&mut self) {
        self.link.send(Box::new(|conn| {
            conn.reset_keep_alive();
        })).ignore();
    }

    pub fn disconnect(&mut self, reason: ChatComponent) {
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

    pub fn build_send_packet<T>(&self, packet: T, func: fn(T, &mut ClientConnection))
    where
        T: Sync + Send + 'static,
    {
        self.link.send(Box::new(move |connection| {
            func(packet, connection)
        })).ignore();
    }

    pub fn execute<T>(&self, task: T)
        where
            T: FnOnce(&mut ClientConnection) + Send + Sync + 'static,
    {
        self.link.send(Box::new(task)).ignore();
    }
}
