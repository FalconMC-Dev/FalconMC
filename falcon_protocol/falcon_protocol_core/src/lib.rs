#![doc = include_str!("../README.md")]

use check::VersionMappings;
use data::parse_mappings;
use generate::generate_output;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_if_dirty, set_dummy};
use quote::quote;
use syn::parse::Parser;

mod check;
mod data;
mod generate;
mod tests;

pub fn protocol_core(input: TokenStream) -> TokenStream {
    set_dummy(quote! {
        pub fn read_packet<B>(buffer: &mut B, packet_id: i32, protocol_version: i32) -> ::std::result::Result<std::option::Option<Box<dyn ::falcon_protocol::Packet>>, ::falcon_packet::ReadError>
        where
            B: ::bytes::Buf,
        {
            unimplemented!()
        }
    });

    let syntax = match parse_mappings.parse2(input) {
        Ok(syntax) => syntax,
        Err(error) => abort!(error.span(), error.to_string()),
    };

    let mappings = VersionMappings::validate_mappings(syntax);

    abort_if_dirty();

    generate_output(mappings)
}
