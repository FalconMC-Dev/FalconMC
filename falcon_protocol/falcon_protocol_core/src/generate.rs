use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Arm, Type};

use crate::check::{LitIntBool, VersionMappings};

type PacketVersionsMap = BTreeMap<LitIntBool, Vec<VersionsToPacket>>;

pub fn generate_output(mappings: VersionMappings) -> TokenStream {
    let mut result = PacketVersionsMap::new();
    let mappings = mappings.packet_to_versions;
    let mappings = BTreeMap::from_iter(mappings.into_iter().map(|(k, v)| (TypeSorted::from(k), v)));
    let mut impls = Vec::new();
    for (packet, mappings) in mappings {
        impls.push(quote!(impl ::falcon_protocol::Packet for #packet {}));
        for (id, versions) in mappings.into_mappings() {
            let entry = result.entry(id).or_default();
            entry.push(VersionsToPacket::new(packet.ty.clone(), versions.into_iter().collect()));
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

#[derive(Debug, Clone)]
struct TypeSorted {
    pub ty: Type,
}

impl PartialEq for TypeSorted {
    fn eq(&self, other: &Self) -> bool {
        self.ty.to_token_stream().to_string() == other.ty.to_token_stream().to_string()
    }
}

impl Eq for TypeSorted {}

impl PartialOrd for TypeSorted {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ty
            .to_token_stream()
            .to_string()
            .partial_cmp(&other.ty.to_token_stream().to_string())
    }
}

impl Ord for TypeSorted {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ty
            .to_token_stream()
            .to_string()
            .cmp(&other.ty.to_token_stream().to_string())
    }
}

impl From<Type> for TypeSorted {
    fn from(value: Type) -> Self { Self { ty: value } }
}

impl ToTokens for TypeSorted {
    fn to_tokens(&self, tokens: &mut TokenStream) { self.ty.to_tokens(tokens); }
}
