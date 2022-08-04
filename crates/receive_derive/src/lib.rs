use std::iter::once;

use falcon_proc_util::ItemListing;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, Item, parse_quote_spanned, Arm, Ident, LitInt, Stmt, ItemFn};

use self::data::PacketData;

mod data;
mod kw;

#[proc_macro]
pub fn falcon_receive(contents: TokenStream) -> TokenStream {
    let mut contents = parse_macro_input!(contents as ItemListing);

    let (packet_data, error): (Vec<PacketData>, Option<syn::Error>) = contents.content
        .iter_mut()
        .filter_map(|item| {
            match item {
                Item::Struct(ref mut item) => {
                    PacketData::parse_packet(item).transpose()
                },
                _ => None,
            }
        })
        .fold((vec![], None), |(mut res, mut err), item| {
            match item {
                Ok(item) => res.push(item),
                Err(error) => {
                    match err {
                        None => err = Some(error),
                        Some(ref mut err) => err.combine(error),
                    }
                }
            }
            (res, err)
        });

    let mut result = contents.into_token_stream();

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
        let packet_ident = data.struct_name();
        let span = packet_ident.span();
        let match_arms: Vec<Arm> = data.mappings().versions()
            .map(|(packet_id, versions)| parse_quote_spanned! {span=>
                #packet_id => {
                    match protocol_id {
                        #(#versions)|* => ::falcon_core::network::packet::PacketDecode::from_buf(buffer)?,
                        _ => return Ok(None),
                    }
                }
            }).collect();

        let fn_body = generate_fn_body(packet_ident, data.mappings().is_exclude(), match_arms);

        let fn_item: ItemFn = parse_quote_spanned! {span=>
            pub fn falcon_process_packet<R, D, L>(packet_id: i32, buffer: &mut R, connection: &mut L) -> ::core::result::Result<::core::option::Option<()>, ::falcon_core::error::FalconCoreError>
            where
                R: ::falcon_core::network::buffer::PacketBufferRead,
                D: ::falcon_core::network::connection::ConnectionDriver,
                L: ::falcon_core::network::connection::ConnectionLogic<D>,
            {
                let protocol_id = connection.driver().handler_state().protocol_id();
                let packet: #packet_ident = { #(#fn_body)* };
                let packet_name = ::falcon_core::network::packet::PacketHandler::get_name(&packet);
                let span = ::tracing::trace_span!("handle_packet", %packet_name);
                let _enter = span.enter();
                Ok(Some(::falcon_core::network::packet::PacketHandler::handle_packet(packet, connection)?))
            }
        };
        result.extend(once(fn_item.into_token_stream()));
    }

    result
}

pub(crate) fn generate_fn_body(packet_ident: &Ident, exclude: Option<&LitInt>, match_arms: Vec<Arm>)-> Vec<Stmt> {
    let span = packet_ident.span();
    if let Some(version) = exclude {
        if match_arms.is_empty() {
            parse_quote_spanned! {span=>
                match packet_id {
                    #version => ::falcon_core::network::packet::PacketDecode::from_buf(buffer)?,
                    _ => return Ok(None),
                }
            }
        } else {
            parse_quote_spanned! {span=>
                match packet_id {
                    #(#match_arms,)*
                    #version => ::falcon_core::network::packet::PacketDecode::from_buf(buffer)?,
                    _ => return Ok(None),
                }
            }
        }
    } else if match_arms.is_empty() {
        parse_quote_spanned! {span=>
            compile_error!("no version mappings provided on a \"falcon_packet\" struct")
        }
    } else {
        parse_quote_spanned! {span=>
            match packet_id {
                #(#match_arms,)*
                _ => return Ok(None),
            }
        }
    }
}
