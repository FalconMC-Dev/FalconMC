use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;

use libloading::{Library, Symbol};
use once_cell::sync::Lazy;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_default_protocol::DefaultProtocol;

use crate::error::{PluginProtocolError, Result};
use crate::FalconPlugin;

pub static PROTOCOL_MANAGER: Lazy<ProtocolPluginManager> = Lazy::new(|| {
    let mut manager = ProtocolPluginManager::new();
    manager.load_plugins();
    manager
});

#[derive(Default)]
pub struct ProtocolPluginManager {
    plugins: Vec<(i32, Box<dyn FalconPlugin>)>,
    loaded_libraries: Vec<Library>,
}

impl ProtocolPluginManager {
    pub fn new() -> ProtocolPluginManager {
        Default::default()
    }

    #[tracing::instrument(skip(self))]
    pub(crate) unsafe fn load_plugin<P: AsRef<OsStr> + Debug>(
        &mut self,
        filename: P,
    ) -> Result<()> {
        debug!("Loading plugin...");
        type PluginCreate = unsafe fn() -> *mut dyn FalconPlugin;

        let lib = Library::new(filename.as_ref()).map_err(|_| PluginProtocolError::LibraryLoadingError(
            filename.as_ref().to_os_string(),
            String::from("File was not recognized as plugin library!"),
        ))?;
        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();
        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create").map_err(|_| PluginProtocolError::LibraryLoadingError(
            filename.as_ref().to_os_string(),
            String::from("The `_plugin_create` symbol wasn't found."),
        ))?;
        let boxed_raw = constructor();

        let plugin = Box::from_raw(boxed_raw);
        debug!(name = plugin.name(), "Plugin loaded!");
        plugin.on_protocol_load();
        self.plugins.push((plugin.get_priority(), plugin));
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn load_plugins(&mut self) {
        if let Ok(paths) = fs::read_dir("./protocols/") {
            for path in paths {
                match path.map_err(|e| format!("Something went wrong when loading from `./protocols/`: {}", e)) {
                    Ok(entry) => {
                        match entry.file_type().map_err(|e| format!("Something went wrong when loading from `./protocols/`, aborted entry '{:?}' due to {}", entry.path(), e)) {
                            Ok(file_type) => {
                                if file_type.is_file() {
                                    if let Err(error) = unsafe { self.load_plugin(entry.path()) } {
                                        warn!("Couldn't load '{:?}' due to '{}'.", entry.path(), error);
                                    }
                                }
                            },
                            Err(ref error) => error!("Encountered error: {}", error),
                        }
                    },
                    Err(ref error) => error!("Encountered error: {}", error),
                }
            }
        }
        self.plugins.sort_by_key(|(priority, _)| *priority);
        info!(len = self.plugins.len(), "Loaded all plugins!");
    }

    #[tracing::instrument(skip(self, buffer, connection))]
    pub fn process_packet<R: PacketBufferRead, C: MinecraftConnection>(
        &self,
        packet_id: i32,
        buffer: &mut R,
        connection: &mut C,
    ) -> Result<Option<()>> {
        let mut found = false;
        // first evaluate default protocol
        match DefaultProtocol::process_packet(packet_id, buffer, connection) {
            Ok(Some(_)) => found = true,
            Err(error) => return Err(error.into()),
            _ => {}
        }
        // then propagate to plugins
        for (_, factory) in &self.plugins {
            trace!(plugin_name = factory.name(), "Firing process_packet()!");
            match factory.process_packet(packet_id, buffer, connection) {
                Ok(Some(_)) => found = true,
                Err(error) => return Err(error),
                _ => {}
            }
        }
        if found {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn unload(&mut self) {
        debug!("Unloading plugins!");

        for (_, plugin) in self.plugins.drain(..) {
            trace!(plugin_name = plugin.name(), "Firing on_plugin_unload()!");
            plugin.on_protocol_unload();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }
    }
}

impl Drop for ProtocolPluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.loaded_libraries.is_empty() {
            self.unload();
        }
    }
}
