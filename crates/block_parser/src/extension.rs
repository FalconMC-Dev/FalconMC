use std::path::PathBuf;
use convert_case::{Case, Casing};
use linked_hash_map::LinkedHashMap;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Token;

pub fn generate_function(name: String) {
    let mut work_dir = PathBuf::from(String::from(env!("CARGO_MANIFEST_DIR")) + "/raw_data");
    work_dir.push(format!("{}.json", name));
    let data: LinkedHashMap<String, bool> = serde_json::from_str(&std::fs::read_to_string(work_dir).unwrap()).unwrap();

    let false_count = data.iter().filter(|(_, value)| !**value).count();
    let (smallest, val) = if false_count + false_count > data.len() {
        (quote!(), true)
    } else {
        let token = Token![!](Span::mixed_site());
        (quote!(#token), false)
    };

    let function_arms: Vec<TokenStream> = data
        .into_iter()
        .filter(|(_, value)| value == &val)
        .map(|(name, _)| {
            let block_name = format_ident!("{}", name.split('.').last().unwrap().to_case(Case::Pascal));
            quote!(Blocks::#block_name)
        }).collect();

    let fn_ident = format_ident!("{}", name);
    let function_file = quote!(
        pub fn #fn_ident(block: &Blocks) -> bool {
            #smallest
            matches!(
                block,
                #(#function_arms)|*
            )
        }
    );

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(format!("{}.rs", name));
    std::fs::write(&path, rustfmt_wrapper::rustfmt(function_file).unwrap()).unwrap();
}