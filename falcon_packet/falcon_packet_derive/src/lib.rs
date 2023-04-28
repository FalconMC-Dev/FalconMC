#![doc = include_str!("../README.md")]

use falcon_packet_core::{packet_core, packet_read, packet_size, packet_write};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn packet(input: TokenStream) -> TokenStream { packet_core(input.into()).into() }

#[proc_macro_error]
#[proc_macro]
pub fn read(input: TokenStream) -> TokenStream { packet_read(input.into()).into() }
#[proc_macro_error]
#[proc_macro]
pub fn write(input: TokenStream) -> TokenStream { packet_write(input.into()).into() }
#[proc_macro_error]
#[proc_macro]
pub fn size(input: TokenStream) -> TokenStream { packet_size(input.into()).into() }
