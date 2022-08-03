
use std::iter::once;

use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{Item, parse_macro_input, Arm, parse_quote_spanned, Stmt, Ident, LitInt, ItemFn, LitStr, Expr};
use crate::data::PacketData;

use crate::util::ItemListing;

mod data;
mod kw;
mod util;

#[proc_macro]
pub fn falcon_send(contents: TokenStream) -> TokenStream {
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
        result.extend(once(generate_send(&data).into_token_stream()));
        if let Some(batch_name) = data.batch_name() {
            result.extend(once(generate_batched(&data, batch_name).into_token_stream()));
        }
    }
    result
}

pub(crate) fn generate_send(data: &PacketData) -> ItemFn {
    let packet_ident = data.struct_name();
    let span = data.struct_name().span();
    let match_arms: Vec<Arm> = data.mappings().versions()
        .map(|(packet_id, versions)| parse_quote_spanned! {span=>
            #(#versions)|* => {
                #packet_id
            }
        }).collect();

    let fn_body = generate_fn_body(
        packet_ident,
        data.mappings().is_exclude(),
        match_arms,
        parse_quote_spanned! {span=> connection.driver().handler_state().protocol_id()},
        parse_quote_spanned! {span=> false}
    );

    let fn_name = data.fn_name();
    let fn_name = Ident::new(&fn_name.value(), fn_name.span());
    let write: Stmt = parse_quote_spanned! {span=>
        connection.driver_mut().send_packet(packet_id, &packet);
    };

    parse_quote_spanned! {fn_name.span()=>
        pub fn #fn_name<T, D, L>(packet: &mut Option<T>, connection: &mut L) -> bool
        where
            #packet_ident: ::std::convert::From<T>,
            D: ::falcon_core::network::connection::ConnectionDriver,
            L: ::falcon_core::network::connection::ConnectionLogic<D>,
        {
            if packet.is_none() {
                return false;
            }
            let packet_id = { #(#fn_body)* };
            let packet: #packet_ident = packet.take().unwrap().into();
            #write
            true
        }
    }
}

pub(crate) fn generate_batched(data: &PacketData, batch: &LitStr) -> ItemFn {
    let packet_ident = data.struct_name();
    let span = data.struct_name().span();
    let match_arms: Vec<Arm> = data.mappings().versions()
        .map(|(packet_id, versions)| parse_quote_spanned! {span=>
            #(#versions)|* => {
                #packet_id
            }
        }).collect();

    let fn_body = generate_fn_body(
        packet_ident,
        data.mappings().is_exclude(),
        match_arms,
        parse_quote_spanned! {span=> protocol_id},
        parse_quote_spanned! {span=> None}
    );

    let fn_name = Ident::new(&batch.value(), batch.span());
    let write: Stmt = parse_quote_spanned! {span=>
        ::falcon_core::network::packet::PacketEncode::to_buf(&packet, &mut buffer);
    };

    parse_quote_spanned! {fn_name.span()=>
        pub fn #fn_name<T>(packet: &mut Option<T>, protocol_id: i32) -> Option<::bytes::Bytes>
        where
            #packet_ident: ::std::convert::From<T>,
        {
            if packet.is_none() {
                return None;
            }
            let mut buffer = ::bytes::BytesMut::new();
            let packet_id = { #(#fn_body)* };
            ::falcon_core::network::buffer::PacketBufferWrite::write_var_i32(&mut buffer, packet_id);
            let packet: #packet_ident = packet.take().unwrap().into();
            #write
            Some(buffer.freeze())
        }
    }
}

pub(crate) fn generate_fn_body(packet_ident: &Ident, exclude: Option<&LitInt>, match_arms: Vec<Arm>, protocol_expr: Expr, default: Expr) -> Vec<Stmt> {
    let span = packet_ident.span();
    if let Some(version) = exclude {
        if match_arms.is_empty() {
            parse_quote_spanned! {span=>
                #version
            }
        } else {
            parse_quote_spanned! {span=>
                let protocol_version = #protocol_expr;
                match protocol_version {
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
            let protocol_version = #protocol_expr;
            match protocol_version {
                #(#match_arms,)*
                _ => return #default,
            }
        }
    }
}

