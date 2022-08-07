use std::iter::once;

use falcon_proc_util::{ErrorCatcher, ItemListing};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, parse_quote_spanned, Arm, Item, ItemFn};

use self::data::PacketData;
use self::util::ReceiveMatchMappings;

mod data;
mod kw;
mod util;

#[proc_macro]
pub fn falcon_receive(contents: TokenStream) -> TokenStream {
    let mut contents = parse_macro_input!(contents as ItemListing);

    let (packet_data, error): (ReceiveMatchMappings, ErrorCatcher) = contents
        .content
        .iter_mut()
        .filter_map(|item| match item {
            Item::Struct(ref mut item) => PacketData::parse_packet(item).transpose(),
            _ => None,
        })
        .fold(
            (ReceiveMatchMappings::new(), ErrorCatcher::new()),
            |(mut res, mut err), item| {
                match item {
                    Ok(item) => {
                        let (exclude, mappings) = item.mappings().to_inner();
                        let name =
                            Some(item.struct_name().clone()).and_then(|n| exclude.map(|v| (v, n)));
                        err.extend_error(
                            res.add_packet(item.struct_name().clone(), (name, mappings)),
                        );
                    }
                    Err(error) => err.add_error(error),
                }
                (res, err)
            },
        );

    let mut result = contents.into_token_stream();

    if let Err(error) = error.emit() {
        result.extend(once(error.to_compile_error()));
    } else {
        result.extend(once(generate(packet_data).into_token_stream()));
    }

    result.into()
}

pub(crate) fn generate(data: ReceiveMatchMappings) -> ItemFn {
    let match_arms: Vec<Arm> = data.mappings.into_iter()
        .map(|(id, mappings)| {
            let packet_id = id.packet_id;
            match id.exclude {
                Some(struct_name) => parse_quote_spanned! {struct_name.span()=>
                    #packet_id => {
                        let packet: #struct_name = ::falcon_core::network::packet::PacketDecode::from_buf(buffer)?;
                        let packet_name = ::falcon_core::network::packet::PacketHandler::get_name(&packet);
                        let span = ::tracing::trace_span!("handle_packet", %packet_name);
                        let _enter = span.enter();
                        Ok(Some(::falcon_core::network::packet::PacketHandler::handle_packet(packet, connection)?))
                    }
                },
                None => {
                    let inner_arms: Vec<Arm> = mappings.versions.into_iter()
                        .map(|(struct_name, versions)| {
                            let versions = versions.iter().map(|(v, _)| v);
                            parse_quote_spanned! {struct_name.span()=>
                                #(#versions)|* => {
                                    let packet: #struct_name = ::falcon_core::network::packet::PacketDecode::from_buf(buffer)?;
                                    let packet_name = ::falcon_core::network::packet::PacketHandler::get_name(&packet);
                                    let span = ::tracing::trace_span!("handle_packet", %packet_name);
                                    let _enter = span.enter();
                                    Ok(Some(::falcon_core::network::packet::PacketHandler::handle_packet(packet, connection)?))
                                }
                            }
                        }).collect();
                    parse_quote! {
                        #packet_id => {
                            match protocol_id {
                                #(#inner_arms,)*
                                _ => Ok(None),
                            }
                        }
                    }
                }
            }
        }).collect();

    parse_quote! {
        pub fn falcon_process_packet<R>(packet_id: i32, buffer: &mut R, connection: &mut ::falcon_logic::connection::FalconConnection) -> ::core::result::Result<::core::option::Option<()>, ::falcon_core::error::FalconCoreError>
        where
            R: ::falcon_core::network::buffer::PacketBufferRead,
        {
            use ::falcon_core::network::connection::ConnectionLogic;
            let protocol_id = connection.handler_state().protocol_id();
            match packet_id {
                #(#match_arms)*
                _ => Ok(None)
            }
        }
    }
}
