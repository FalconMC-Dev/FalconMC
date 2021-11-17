use crate::network::buffer::PacketBufferWrite;
use crossbeam::channel::Sender;

use crate::network::PacketHandlerState;
use crate::server::McTask;

pub type ConnectionTask = dyn Fn(&mut dyn MinecraftConnection) -> () + Send + Sync;

pub trait MinecraftConnection {
    fn get_handler_state(&self) -> &PacketHandlerState;

    fn get_handler_state_mut(&mut self) -> &mut PacketHandlerState;

    fn get_server_link_mut(&mut self) -> &mut Sender<Box<McTask>>;

    fn send_packet(
        &mut self,
        packet_id: i32,
        buffer_write: Box<dyn FnOnce(&mut dyn PacketBufferWrite)>,
    );

    fn disconnect(&mut self, reason: String); // TODO: change into ChatComponent
}
