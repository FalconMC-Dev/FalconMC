use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedSender;

use crate::network::packet::PacketEncode;
use crate::network::PacketHandlerState;
use crate::server::McTask;

pub type ConnectionTask = dyn FnOnce(&mut dyn MinecraftConnection) + Send + Sync;

pub trait MinecraftConnection {
    fn get_address(&self) -> &SocketAddr;

    fn get_handler_state(&self) -> &PacketHandlerState;

    fn get_handler_state_mut(&mut self) -> &mut PacketHandlerState;

    fn get_server_link_mut(&mut self) -> &mut UnboundedSender<Box<McTask>>;

    fn get_connection_link(&self) -> UnboundedSender<Box<ConnectionTask>>;

    fn send_packet(&mut self, packet_id: i32, packet_out: &dyn PacketEncode);

    fn reset_keep_alive(&mut self);

    fn disconnect(&mut self, reason: String); // TODO: change into ChatComponent
}
