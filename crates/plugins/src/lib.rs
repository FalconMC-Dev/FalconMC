#[macro_use]
extern crate tracing;

use error::Result;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::ClientConnection;

pub mod error;
mod macros;
pub mod manager;

use abi_stable::{declare_root_module_statics, package_version_strings, StableAbi};
use abi_stable::library::RootModule;
use abi_stable::sabi_types::VersionStrings;
use abi_stable::std_types::RStr;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref="FalconLib_Ref")))]
#[sabi(missing_field(panic))]
pub struct FalconLib {
    pub new: extern "C" fn() -> i32,
}

//pub type PluginBox = FalconPlugin_TO<'static, RBox<()>>;

//#[abi_stable::sabi_trait]
pub trait FalconPlugin: Send + Sync {
    fn name(&self) -> RStr<'static>;

    fn on_protocol_load(&self) {}

    fn on_protocol_unload(&self) {}

    /// Returns the importance of this `FalconPlugin`'s packet querying, lower numbers are more important.
    ///
    /// 0-1-2-3 are reserved for special implementations.
    fn get_priority(&self) -> i32 {
        4
    }

    fn process_packet(
        &self,
        packet_id: i32,
        buffer: &mut dyn PacketBufferRead,
        connection: &mut ClientConnection,
    ) -> Result<Option<()>>;
}

impl RootModule for FalconLib_Ref {
    // The name of the dynamic library
    const BASE_NAME: &'static str = "falcon_plugin";
    // The name of the library for logging and similars
    const NAME: &'static str = "falcon_plugin";
    // The version of this plugin's crate
    const VERSION_STRINGS: VersionStrings = package_version_strings!();

    // Implements the `RootModule::root_module_statics` function, which is the
    // only required implementation for the `RootModule` trait.
    declare_root_module_statics!{FalconLib_Ref}
}