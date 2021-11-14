use std::ffi::OsStr;
use std::fs;

use libloading::{Library, Symbol};
use once_cell::sync::Lazy;

use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;

use crate::errors::*;
use crate::ProtocolPlugin;

pub static PROTOCOL_MANAGER: Lazy<ProtocolPluginManager> = Lazy::new(|| {
    let mut manager = ProtocolPluginManager::new();
    manager.load_plugins();
    manager
});

pub struct ProtocolPluginManager {
    plugins: Vec<(i32, Box<dyn ProtocolPlugin>)>,
    loaded_libraries: Vec<Library>,
}

impl ProtocolPluginManager {
    pub fn new() -> ProtocolPluginManager {
        ProtocolPluginManager {
            plugins: vec![],
            loaded_libraries: vec![],
        }
    }

    pub(crate) unsafe fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<()> {
        debug!("Loading plugin '{:?}'", filename.as_ref());
        type PluginCreate = unsafe fn() -> *mut dyn ProtocolPlugin;

        let lib = Library::new(filename.as_ref()).chain_err(|| {
            ErrorKind::LibraryLoadingError(
                filename.as_ref().to_os_string(),
                String::from("Unable to load plugin from disk!"),
            )
        })?;
        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();
        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create").chain_err(|| {
            ErrorKind::LibraryLoadingError(
                filename.as_ref().to_os_string(),
                String::from("The `_plugin_create` symbol wasn't found."),
            )
        })?;
        let boxed_raw = constructor();

        let plugin = Box::from_raw(boxed_raw);
        debug!("Loaded plugin: {}", plugin.name());
        plugin.on_protocol_load();
        self.plugins.push((plugin.get_priority(), plugin));
        Ok(())
    }

    pub(crate) fn load_plugins(&mut self) {
        if let Ok(paths) = fs::read_dir("./protocols/") {
            for path in paths {
                match path.chain_err(|| "Something went wrong when loading from `./protocols/`") {
                    Ok(entry) => {
                        match entry.file_type().chain_err(|| format!("Something went wrong when loading from `./protocols/`, aborted entry '{:?}'", entry.path())) {
                            Ok(file_type) => {
                                if file_type.is_file() {
                                    if let Err(error) = unsafe { self.load_plugin(entry.path()) } {
                                        warn!("Couldn't load '{:?}' due to '{}'.", entry.path(), error);
                                    }
                                }
                            },
                            Err(ref error) => print_error!(error),
                        }
                    },
                    Err(ref error) => print_error!(error),
                }
            }
        } else {
            warn!("No protocols are found, the application will be useless!");
        }
        self.plugins.sort_by_key(|(priority, _)| *priority);
    }

    pub fn process_packet(
        &self,
        packet_id: i32,
        buffer: &mut dyn PacketBufferRead,
        connection: &mut dyn MinecraftConnection,
    ) -> Result<Option<()>> {
        let mut found = false;
        for (_, factory) in &self.plugins {
            trace!("Firing read_packet for {}", factory.name());
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

    pub(crate) fn unload(&mut self) {
        debug!("Unloading plugins!");

        for (_, plugin) in self.plugins.drain(..) {
            trace!("Firing on_plugin_unload for {:?}", plugin.name());
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
