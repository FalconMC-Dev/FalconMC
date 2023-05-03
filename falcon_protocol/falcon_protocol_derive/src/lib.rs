#![doc = include_str!("../README.md")]

use falcon_protocol_core::protocol_core;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn packets(input: TokenStream) -> TokenStream { protocol_core(input.into()).into() }
