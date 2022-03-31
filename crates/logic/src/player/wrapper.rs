use mc_chat::ChatComponent;

pub trait PlayerLogic {
    fn disconnect(&mut self, reason: ChatComponent);

    fn send_keep_alive(&self);
}