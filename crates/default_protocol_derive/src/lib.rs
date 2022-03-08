#[macro_use]
extern crate quote;

use proc_macro::TokenStream as TokenStream2;
use proc_macro2::TokenStream;
use syn::parse::Nothing;
use syn::{Item, ItemMod, parse_macro_input};

mod packet_mod;

#[proc_macro_attribute]
pub fn packet_module(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let _ = parse_macro_input!(attr as Nothing);
    let module = parse_macro_input!(item as ItemMod);

    if let Some((_, ref content)) = module.content {
        for item in content {
            if let Item::Struct(ref data) = item {

            }
        }
        println!("Content: {:#?}", content);
    }

    quote!(
        #module
    ).into()
}
