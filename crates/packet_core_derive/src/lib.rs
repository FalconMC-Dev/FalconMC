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

#[proc_macro_derive(PacketWrite, attributes(falcon))]
pub fn derive_packet_write(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    match implement_write(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

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
