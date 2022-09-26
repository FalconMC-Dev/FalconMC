use std::iter::once;

use proc_macro::TokenStream;

use crate::data::PacketData;
use quote::ToTokens;
use syn::{
    parse_macro_input, parse_quote_spanned, Arm, Ident, Item, ItemFn, ItemMod, LitInt, Stmt,
};

mod data;
mod kw;

#[proc_macro_attribute]
pub fn falcon_send(_attr: TokenStream, contents: TokenStream) -> TokenStream {
    let mut contents = parse_macro_input!(contents as ItemMod).content.unwrap().1;

    let (packet_data, error): (Vec<PacketData>, Option<syn::Error>) = contents
        .iter_mut()
        .filter_map(|item| match item {
            Item::Struct(ref mut item) => PacketData::parse_packet(item).transpose(),
            _ => None,
        })
        .fold((vec![], None), |(mut res, mut err), item| {
            match item {
                Ok(item) => res.push(item),
                Err(error) => match err {
                    None => err = Some(error),
                    Some(ref mut err) => err.combine(error),
                },
            }
            (res, err)
        });

    let mut result = proc_macro2::TokenStream::new();
    result.extend(contents.into_iter().map(|i| i.to_token_stream()));

    if let Some(error) = error {
        result.extend(once(error.to_compile_error()));
    } else {
        result.extend(once(generate(packet_data)));
    }

    result.into()
}

pub(crate) fn generate(data: Vec<PacketData>) -> proc_macro2::TokenStream {
    let mut result = proc_macro2::TokenStream::new();
    for data in data {
        result.extend(once(generate_send(&data).into_token_stream()));
    }
    result
}

pub(crate) fn generate_send(data: &PacketData) -> ItemFn {
    let packet_ident = data.struct_name();
    let span = data.struct_name().span();
    let match_arms: Vec<Arm> = data
        .mappings()
        .versions()
        .map(|(packet_id, versions)| {
            parse_quote_spanned! {span=>
                #(#versions)|* => {
                    #packet_id
                }
            }
        })
        .collect();

    let fn_body = generate_fn_body(packet_ident, data.mappings().is_exclude(), match_arms);

    let fn_name = data.fn_name();
    let fn_name = Ident::new(&fn_name.value(), fn_name.span());

    parse_quote_spanned! {fn_name.span()=>
        pub fn #fn_name<T, B>(
            packet: &mut Option<T>,
            buffer: &mut B,
            _protocol: i32,
        ) -> Result<bool, ::falcon_packet_core::WriteError>
        where
            #packet_ident: ::std::convert::From<T>,
            B: ::falcon_packet_core::special::BufRes,
        {
            if packet.is_none() {
                return Ok(false);
            }
            let packet_id = ::falcon_packet_core::VarI32::from(#(#fn_body)*);
            let packet: #packet_ident = packet.take().unwrap().into();
            buffer.reserve(
                ::falcon_packet_core::PacketSize::size(&packet_id)
                    + ::falcon_packet_core::PacketSize::size(&packet)
            );
            ::falcon_packet_core::PacketWrite::write(
                &packet_id,
                buffer,
            )?;
            ::falcon_packet_core::PacketWrite::write(
                &packet,
                buffer,
            )?;
            Ok(true)
        }
    }
}

pub(crate) fn generate_fn_body(
    packet_ident: &Ident,
    exclude: Option<&LitInt>,
    match_arms: Vec<Arm>,
) -> Vec<Stmt> {
    let span = packet_ident.span();
    if let Some(version) = exclude {
        if match_arms.is_empty() {
            parse_quote_spanned! {span=>
                #version
            }
        } else {
            parse_quote_spanned! {span=>
                match _protocol {
                    #(#match_arms,)*
                    _ => #version,
                }
            }
        }
    } else if match_arms.is_empty() {
        parse_quote_spanned! {span=>
            compile_error!("no version mappings provided on a \"falcon_packet\" struct")
        }
    } else {
        parse_quote_spanned! {span=>
            match _protocol {
                #(#match_arms,)*
                _ => return Ok(false),
            }
        }
    }
}
