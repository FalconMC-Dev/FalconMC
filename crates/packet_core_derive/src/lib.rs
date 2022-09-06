use crate::write::implement_write;
use syn::{parse_macro_input, ItemStruct};

pub(crate) mod attributes;
pub(crate) mod kw;
mod read;
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
