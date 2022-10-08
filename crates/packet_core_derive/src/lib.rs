use read::implement_read;
use size::implement_size;
use syn::{parse_macro_input, ItemStruct};

use crate::write::implement_write;

pub(crate) mod attributes;
pub(crate) mod kw;
mod read;
mod size;
pub(crate) mod util;
mod write;

/// Derive macro for [`PacketWrite`](falcon_packet_core::PacketWrite).
///
/// Currently, this trait can only be derived for structs with named fields. The struct must
/// implement [`PacketSize`](falcon_packet_core::PacketSize) and each field's type must
/// implement [`PacketWrite`](falcon_packet_core::PacketWrite).
///
/// The following field attributes are available:
///
/// name | argument(s) | function
/// --- | --- | ---
/// `var32` | | Write the field using [varint format](https://wiki.vg/Protocol#VarInt_and_VarLong).
/// `var64` | | Write the field using [varlong format](https://wiki.vg/Protocol#VarInt_and_VarLong).
/// `array` | | Required for any [`array`] except byte arrays.
/// `bytes` | | Required for any [`AsRef<[u8]>`](AsRef).
/// `into` | `= "type"` | Before writing, first conver the field into the given type.
/// `convert` | `= "type"` | Before writing, first convert the field into the given type. (overlaps with [`PacketRead`](falcon_packet_core_derive::PacketRead))
/// `nbt` | | Write as nbt data using [fastnbt](::fastnbt).
/// `string` | `= length` | Required for any [`AsRef<str>`](AsRef). The given length should be the maximum length allowed by the protocol.
/// `to_string` | `= length` | Required for any [`ToString`]. The given length should be the maximum length allowed by the prococol.
/// `vec` | | Required for any type that implements [`IntoIterator`].
///
/// # Example
/// ```ignore
/// use falcon_packet_core::PacketWrite;
///
/// #[derive(PacketSize, PacketWrite)]
/// struct MyStruct {
///     #[falcon(var32)]
///     id: i32,
///     number: i32,
///     #[falcon(string = 30)]
///     text: String,
///     #[falcon(to_string = 20)]
///     fancy_string: String,
///     #[falcon(vec)]
///     uuids: Vec<Uuid>,
/// }
/// ```
///
#[rustfmt::skip]
#[allow(rustdoc::broken_intra_doc_links)]
#[proc_macro_derive(PacketWrite, attributes(falcon))]
pub fn derive_packet_write(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    match implement_write(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Derive macro for [`PacketSize`](falcon_packet_core::PacketSize).
///
/// Currently, this trait can only be derived for structs with named fields. Each field's
/// type must implement [`PacketSize`](falcon_packet_core::PacketSize).
///
/// The following field attributes are available:
///
/// name | argument(s) | function
/// --- | --- | ---
/// `var32` | | Write the field using [varint format](https://wiki.vg/Protocol#VarInt_and_VarLong).
/// `var64` | | Write the field using [varlong format](https://wiki.vg/Protocol#VarInt_and_VarLong).
/// `array` | | Required for any [`array`] except byte arrays.
/// `bytes` | | Required for any [`AsRef<[u8]>`](AsRef).
/// `into` | `= "type"` | Before writing, first conver the field into the given type.
/// `convert` | `= "type"` | Before writing, first convert the field into the given type. (overlaps with [`PacketRead`](falcon_packet_core_derive::PacketRead))
/// `nbt` | | Write as nbt data using [fastnbt](::fastnbt).
/// `string` | `= length` | Required for any [`AsRef<str>`](AsRef). The given length should be the maximum length allowed by the protocol.
/// `to_string` | `= length` | Required for any [`ToString`]. The given length should be the maximum length allowed by the prococol.
/// `vec` | | Required for any type that implements [`IntoIterator`].
///
/// # Example
/// ```ignore
/// use falcon_packet_core::PacketSize;
///
/// #[derive(PacketSize)]
/// struct MyStruct {
///     #[falcon(var32)]
///     id: i32,
///     number: i32,
///     #[falcon(string = 30)]
///     text: String,
///     #[falcon(to_string = 20)]
///     fancy_string: String,
///     #[falcon(vec)]
///     uuids: Vec<Uuid>,
/// }
/// ```
///
#[rustfmt::skip]
#[allow(rustdoc::broken_intra_doc_links)]
#[proc_macro_derive(PacketSize, attributes(falcon))]
pub fn derive_packet_size(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    match implement_size(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(PacketRead, attributes(falcon))]
pub fn derive_packet_read(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    match implement_read(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
