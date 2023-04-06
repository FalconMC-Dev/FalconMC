#![doc = include_str!("../README.md")]

use falcon_packet_core::packet_core;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn packet(input: TokenStream) -> TokenStream {
    packet_core(input.into()).into()
}
