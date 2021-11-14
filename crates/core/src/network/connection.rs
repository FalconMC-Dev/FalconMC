use crossbeam::channel::Sender;

use crate::network::PacketHandlerState;
use crate::server::McTask;

pub type ConnectionTask = dyn Fn(&mut dyn MinecraftConnection) -> () + Send + Sync;

pub trait MinecraftConnection {
    fn get_handler_state(&self) -> &PacketHandlerState;

    fn get_handler_state_mut(&mut self) -> &mut PacketHandlerState;

    fn get_server_link_mut(&mut self) -> &mut Sender<Box<McTask>>;
}
