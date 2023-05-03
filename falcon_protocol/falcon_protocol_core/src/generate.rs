use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Arm, Type};

use crate::check::{LitIntBool, VersionMappings};

type PacketVersionsMap = HashMap<LitIntBool, Vec<VersionsToPacket>>;

pub fn generate_output(mappings: VersionMappings) -> TokenStream {
    let mut result = PacketVersionsMap::new();
    let mappings = mappings.packet_to_versions;
    let mut impls = Vec::new();
    for (packet, mappings) in mappings {
        impls.push(quote!(impl ::falcon_protocol::Packet for #packet {}));
        for (id, versions) in mappings.into_mappings() {
            let entry = result.entry(id).or_default();
            entry.push(VersionsToPacket::new(packet.clone(), versions.into_iter().collect()));
        }
    }

    let arms = result.iter().map(|(id, mappings)| {
        quote! {
            #id => {
                match protocol_version {
                    #(#mappings)*
                    _ => None,
                }
            }
        }
    });

    quote! {
        #(#impls)*
        pub fn read_packet<B>(buffer: &mut B, packet_id: i32, protocol_version: i32) -> ::std::result::Result<std::option::Option<Box<dyn ::falcon_protocol::Packet>>, ::falcon_packet::ReadError>
        where
            B: ::bytes::Buf,
        {
            Ok(match packet_id {
                #(#arms)*
                _ => None,
            })
        }
    }
}

struct VersionsToPacket {
    ty: Type,
    versions: Vec<LitIntBool>,
}

impl VersionsToPacket {
    pub fn new(ty: Type, versions: Vec<LitIntBool>) -> Self { Self { ty, versions } }
}

impl ToTokens for VersionsToPacket {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let packet = &self.ty;
        let versions = &self.versions;
        let arm: Arm = parse_quote! {
            #(#versions)|* => Some(::std::boxed::Box::new(<#packet as ::falcon_packet::PacketRead>::read(buffer)?)),
        };
        arm.to_tokens(tokens);
    }
}
