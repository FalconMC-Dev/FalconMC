#![doc = include_str!("../README.md")]

use check::VersionMappings;
use data::parse_mappings;
use generate::generate_output;
use proc_macro2::TokenStream;
use proc_macro_error::abort_if_dirty;
use syn::parse::Parser;

mod check;
mod data;
mod generate;

pub fn protocol_core(input: TokenStream) -> TokenStream {
    let syntax = match parse_mappings.parse2(input) {
        Ok(syntax) => syntax,
        Err(error) => return error.to_compile_error(),
    };

    let mappings = VersionMappings::validate_mappings(syntax);

    abort_if_dirty();

    generate_output(mappings)
}
