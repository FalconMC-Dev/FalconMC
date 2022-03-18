#[macro_use]
extern crate quote;

use proc_macro::TokenStream as TokenStream2;
use proc_macro2::Span;
use syn::parse::{Nothing, Parse, ParseStream};
use syn::{Item, ItemMod, parse_macro_input, Ident, ItemStruct, Token, Error, LitInt, LitStr};
use syn::punctuated::Punctuated;
use crate::FalconArg::{ExcludeReceive, Outgoing, VersionToId};
use crate::packet_mod::{packet_structs_to_version_outgoing_list, packet_structs_to_version_receive_list};

mod packet_mod;
mod kw;

#[proc_macro_attribute]
pub fn packet_module(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let _ = parse_macro_input!(attr as Nothing);
    let mut module = parse_macro_input!(item as ItemMod);

    if let Some((_, ref mut content)) = module.content {
        let mut packet_structs = Vec::new();
        for item in content.iter_mut() {
            if let Item::Struct(ref mut data) = item {
                match try_into_packet_struct(data) {
                    Ok(packet_struct) => packet_structs.push(packet_struct),
                    Err(error) => {
                        let error_tokens = error.to_compile_error();
                        return quote!(
                            #module
                            #error_tokens
                        ).into();
                    },
                }
            }
        }

        // Receiving
        let version_to_packet_id = packet_structs_to_version_receive_list(&packet_structs);
        let mut match_arms_receive = Vec::new();
        let receive_empty = version_to_packet_id.is_empty();
        for (version, packet_list) in version_to_packet_id {
            let mut inner_match_arms = Vec::new();
            for (struct_name, packet_id) in packet_list {
                let span = struct_name.span();
                inner_match_arms.push(quote_spanned!(span=>
                    #packet_id => Ok(Some(::falcon_core::network::packet::PacketHandler::handle_packet(<#struct_name as ::falcon_core::network::packet::PacketDecode>::from_buf(buffer)?, connection)?))
                ));
            }
            match_arms_receive.push(quote!(
                #version => {
                    match packet_id {
                        #(#inner_match_arms,)*
                        _ => Ok(None),
                    }
                }
            ));
        }
        if !receive_empty {
            content.push(Item::Verbatim(quote!(
                pub fn falcon_process_packet<R, C>(packet_id: i32, buffer: &mut R, connection: &mut C) -> Result<Option<()>, crate::error::DefaultProtocolError>
                where
                    R: ::falcon_core::network::buffer::PacketBufferRead,
                    C: ::falcon_core::network::connection::MinecraftConnection,
                {
                    let protocol_version = connection.handler_state().protocol_id();
                    match protocol_version {
                        #(#match_arms_receive,)*
                        _ => Ok(None)
                    }
                }
            )));
        } else {
            content.push(Item::Verbatim(quote!(
                pub fn falcon_process_packet<R, C>(packet_id: i32, buffer: &mut R, connection: &mut C) -> Result<Option<()>, crate::error::DefaultProtocolError>
                where
                    R: ::falcon_core::network::buffer::PacketBufferRead,
                    C: ::falcon_core::network::connection::MinecraftConnection,
                {
                    Ok(None)
                }
            )));
        }

        // Sending
        let version_to_packet_id = packet_structs_to_version_outgoing_list(&packet_structs);
        let mut functions_outgoing = Vec::new();
        for (packet_ident, (name, packet_list)) in version_to_packet_id {
            let mut inner_match_arms = Vec::new();
            for (version, packet_id) in packet_list {
                let span = packet_ident.span();
                inner_match_arms.push(quote_spanned!(span=>
                    #version => connection.send_packet(#packet_id, &packet)
                ));
            }
            let name_spanned = Ident::new(&format!("falcon_{}", name.value()), name.span());
            functions_outgoing.push(quote!(
                pub fn #name_spanned<T, C>(packet: T, connection: &mut C) -> Option<()>
                where
                    #packet_ident: ::std::convert::From<T>,
                    C: ::falcon_core::network::connection::MinecraftConnection,
                {
                    let packet: #packet_ident = packet.into();
                    let protocol_version = connection.handler_state().protocol_id();
                    match protocol_version {
                        #(#inner_match_arms,)*
                        _ => return None,
                    }
                    Some(())
                }
            ));
        }
        content.push(Item::Verbatim(quote!(
            #(#functions_outgoing)*
        )));
    }

    quote!(
        #module
    ).into()
}

fn try_into_packet_struct(item: &mut ItemStruct) -> syn::Result<PacketStruct> {
    let mut versions: Vec<(i32, Vec<i32>)> = Vec::new();
    let mut outgoing = None;
    let mut incoming = true;
    for attr in &item.attrs {
        if attr.path.is_ident("falcon_packet") {
            let falcon_args = attr.parse_args::<FalconAttrArgs>()?;
            for version_to_packet in &falcon_args.versions {
                let mut temp_versions = Vec::new();
                for version_lit in &version_to_packet.versions {
                    let version = version_lit.base10_parse::<i32>()?;
                    if versions.iter().any(|(_, v)| v.contains(&version)) {
                        return Err(Error::new(version_lit.span(), "A previous assignment is already associated with this protocol version"));
                    } else {
                        temp_versions.push(version);
                    }
                }
                versions.push((version_to_packet.packet_id, temp_versions));
            }
            if let Some((span, name)) = falcon_args.outgoing {
                if outgoing.is_some() {
                    return Err(Error::new(span, "A previous declaration already exists!"))
                } else {
                    outgoing = Some(name)
                }
            }
            if !falcon_args.incoming {
                incoming = false
            }
        }
    }
    item.attrs.retain(|attr| !attr.path.is_ident("falcon_packet"));
    Ok(PacketStruct {
        struct_name: item.ident.clone(),
        versions,
        outgoing,
        incoming,
    })
}

struct PacketStruct {
    struct_name: Ident,
    versions: Vec<(i32, Vec<i32>)>,
    outgoing: Option<LitStr>,
    incoming: bool,
}

struct FalconAttrArgs {
    versions: Vec<VersionToPacketId>,
    outgoing: Option<(Span, LitStr)>,
    incoming: bool,
}

impl Parse for FalconAttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut versions = Vec::new();
        let mut outgoing = None;
        let mut incoming = true;
        let falcon_args = Punctuated::<FalconArg, Token![;]>::parse_terminated(input)?;
        for arg in falcon_args {
            match arg {
                VersionToId(element) => {
                    for version_lit in &element.versions {
                        let version: i32 = version_lit.base10_parse::<i32>()?;
                        if version < 0 {
                            return Err(Error::new(version_lit.span(), "Protocol versions should be non-negative integers"));
                        }
                        if versions.iter().any(|e: &VersionToPacketId| {
                            for v in &e.versions {
                                if let Ok(v) = v.base10_parse::<i32>() {
                                    return v == version;
                                }
                            }
                            false
                        }) {
                            return Err(Error::new(version_lit.span(), "A previous assignment is already associated with this protocol version"));
                        }
                    }
                    versions.push(element);
                }
                Outgoing(name) => outgoing = Some(name),
                ExcludeReceive => incoming = false,
            }
        }
        Ok(FalconAttrArgs {
            versions,
            outgoing,
            incoming,
        })
    }
}

enum FalconArg {
    VersionToId(VersionToPacketId),
    Outgoing((Span, LitStr)),
    ExcludeReceive,
}

impl Parse for FalconArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw::no_receive) {
            input.parse::<kw::no_receive>()?;
            Ok(ExcludeReceive)
        } else if input.peek(kw::outgoing) {
            let outgoing = input.parse::<kw::outgoing>()?;
            input.parse::<Token![=]>()?;
            let spec_name = input.parse::<LitStr>()?;
            Ok(Outgoing((outgoing.span, spec_name)))
        } else {
            match input.parse::<VersionToPacketId>() {
                Ok(data) => Ok(VersionToId(data)),
                Err(err) => Err(Error::new(err.span(), "Unexpected attribute arguments!"))
            }
        }
    }
}

struct VersionToPacketId {
    pub versions: Vec<LitInt>,
    pub packet_id: i32,
}

impl Parse for VersionToPacketId {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let versions: Vec<LitInt> = Punctuated::<LitInt, Token![,]>::parse_separated_nonempty(input)?
            .iter().cloned().collect();
        input.parse::<Token![=]>()?;
        let packet_id_lit = LitInt::parse(input)?;
        let packet_id = packet_id_lit.base10_parse::<i32>()?;
        if packet_id < 0 {
            return Err(Error::new(packet_id_lit.span(), "Packet IDs should be non-negative integers"));
        }
        Ok(VersionToPacketId {
            versions,
            packet_id,
        })
    }
}

