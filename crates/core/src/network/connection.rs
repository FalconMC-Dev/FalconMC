pub type ConnectionTask = dyn Fn(&mut dyn MinecraftConnection) -> () + Send + Sync;

pub trait MinecraftConnection {}
