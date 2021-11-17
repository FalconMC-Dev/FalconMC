//! Part of the Public API of FalconMC

pub mod config;

pub type McTask = dyn Fn(&mut dyn MinecraftServer) -> () + Send + Sync;

pub trait MinecraftServer {}
