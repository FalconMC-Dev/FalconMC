use std::iter::once;

use falcon_proc_util::ErrorCatcher;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Nothing;
use syn::{parse_macro_input, parse_quote, parse_quote_spanned, Arm, Item, ItemFn, ItemMod};

use self::data::PacketData;
use self::util::ReceiveMatchMappings;

mod data;
mod kw;
mod util;

#[proc_macro_attribute]
pub fn falcon_receive(attrs: TokenStream, contents: TokenStream) -> TokenStream {
    let mut contents = parse_macro_input!(contents as ItemMod).content.unwrap().1;
    let _ = parse_macro_input!(attrs as Nothing);

    let (packet_data, error): (ReceiveMatchMappings, ErrorCatcher) = contents
        .iter_mut()
        .filter_map(|item| match item {
            Item::Struct(ref mut item) => PacketData::parse_packet(item).transpose(),
            _ => None,
        })
        .fold((ReceiveMatchMappings::new(), ErrorCatcher::new()), |(mut res, mut err), item| {
            match item {
                Ok(item) => {
                    let (exclude, mappings) = item.mappings().to_inner();
                    let name = Some(item.struct_name().clone()).and_then(|n| exclude.map(|v| (v, n)));
                    err.extend_error(res.add_packet(item.struct_name().clone(), (name, mappings)));
                },
                Err(error) => err.add_error(error),
            }
            (res, err)
        });

    let mut result = proc_macro2::TokenStream::new();
    result.extend(contents.into_iter().map(|i| i.to_token_stream()));

    if let Err(error) = error.emit() {
        result.extend(once(error.to_compile_error()));
    } else {
        result.extend(once(generate(packet_data).into_token_stream()));
    }

    result.into()
}

pub(crate) fn generate(data: ReceiveMatchMappings) -> ItemFn {
    let match_arms: Vec<Arm> = data
        .mappings
        .into_iter()
        .map(|(id, mappings)| {
            let packet_id = id.packet_id;
            match id.exclude {
                Some(struct_name) => parse_quote_spanned! {struct_name.span()=>
                    #packet_id => {
                        let packet: #struct_name = ::falcon_packet_core::PacketRead::read(buffer)?;
                        let packet_name = ::falcon_logic::connection::handler::PacketHandler::get_name(&packet);
                        let span = ::tracing::trace_span!("handle_packet", %packet_name);
                        let _enter = span.enter();
                        ::falcon_logic::connection::handler::PacketHandler::handle_packet(packet, connection)?;
                        Ok(true)
                    }
                },
                None => {
                    let inner_arms: Vec<Arm> = mappings
                        .versions
                        .into_iter()
                        .map(|(struct_name, versions)| {
                            let versions = versions.iter().map(|(v, _)| v);
                            parse_quote_spanned! {struct_name.span()=>
                                #(#versions)|* => {
                                    let packet: #struct_name = ::falcon_packet_core::PacketRead::read(buffer)?;
                                    let packet_name = ::falcon_logic::connection::handler::PacketHandler::get_name(&packet);
                                    let span = ::tracing::trace_span!("handle_packet", %packet_name);
                                    let _enter = span.enter();
                                    ::falcon_logic::connection::handler::PacketHandler::handle_packet(packet, connection)?;
                                    Ok(true)
                                }
                            }
                        })
                        .collect();
                    parse_quote! {
                        #packet_id => {
                            match protocol_id {
                                #(#inner_arms,)*
                                _ => Ok(false),
                            }
                        }
                    }
                },
            }
        })
        .collect();

    parse_quote! {
        pub fn falcon_process_packet<B>(packet_id: i32, buffer: &mut B, connection: &mut ::falcon_logic::connection::FalconConnection) -> ::anyhow::Result<bool>
        where
            B: ::bytes::Buf,
        {
            let protocol_id = connection.handler_state().protocol_id();
            match packet_id {
                #(#match_arms)*
                _ => Ok(false)
            }
        }
    }
}
