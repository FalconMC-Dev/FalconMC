use crate::write::implement_write;
use syn::{parse_macro_input, ItemStruct};

mod read;
mod write;

#[proc_macro_derive(PacketWrite)]
pub fn derive_packet_write(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    match implement_write(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

