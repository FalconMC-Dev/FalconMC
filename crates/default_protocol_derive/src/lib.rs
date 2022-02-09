#[macro_use]
extern crate quote;

use darling::ast::Fields;
use darling::util::SpannedValue;
use darling::{Error, FromField, FromMeta, FromVariant};
use proc_macro::TokenStream as TokenStream2;
use proc_macro2::Ident;
use syn::{parse_macro_input, Attribute, ItemEnum, Type};

#[derive(Debug, FromMeta)]
struct PacketEnumAttr {
    id: i32,
}

#[derive(Debug, FromField)]
struct PacketEnumVariantField {
    ty: syn::Type,
}

#[derive(Debug, FromVariant)]
#[darling(forward_attrs(falcon_packet))]
struct PacketEnumVariant {
    attrs: Vec<Attribute>,
    fields: Fields<PacketEnumVariantField>,
}

#[proc_macro_derive(PacketEnum, attributes(falcon_packet))]
pub fn derive_packet_enum(input: TokenStream2) -> TokenStream2 {
    let item_enum = parse_macro_input!(input as ItemEnum);

    let ident = &item_enum.ident;

    let variants: Result<Vec<SpannedValue<PacketEnumVariant>>, Error> = item_enum
        .variants
        .iter()
        .map(SpannedValue::from_variant)
        .collect();
    let variants = match variants {
        Ok(v) => v,
        Err(e) => {
            return TokenStream2::from(e.write_errors());
        }
    };

    let mut packet_ids: Vec<i32> = Vec::new();
    let mut variant_types: Vec<Ident> = Vec::new();
    for variant in variants.iter() {
        let attr = match variant.attrs.get(0) {
            Some(attr) => attr,
            None => {
                return TokenStream2::from(
                    darling::Error::custom("All variants should expose a Packet Id")
                        .with_span(variant)
                        .write_errors(),
                )
            }
        };
        let meta = match attr.parse_meta() {
            Ok(v) => v,
            Err(e) => return TokenStream2::from(e.into_compile_error()),
        };
        let args: PacketEnumAttr = match PacketEnumAttr::from_meta(&meta) {
            Ok(v) => v,
            Err(e) => return TokenStream2::from(e.write_errors()),
        };
        packet_ids.push(args.id);

        let field_ty = match variant.fields.fields.get(0) {
            Some(v) => v,
            None => {
                return TokenStream2::from(
                    darling::Error::custom("All varaints should contain a type!")
                        .with_span(variant)
                        .write_errors(),
                )
            }
        };
        let ty = match &field_ty.ty {
            Type::Path(v) => match v.path.get_ident() {
                Some(v) => v,
                None => {
                    return TokenStream2::from(
                        darling::Error::custom(
                            "We cannot parse segmented paths (yet), please import these",
                        )
                        .with_span(&field_ty.ty)
                        .write_errors(),
                    )
                }
            },
            _ => {
                return TokenStream2::from(
                    darling::Error::custom("Unexpected type for variant")
                        .with_span(&field_ty.ty)
                        .write_errors(),
                )
            }
        };
        variant_types.push(ty.clone());
    }
    let variant_idents: Vec<Ident> = item_enum.variants.iter().map(|v| v.ident.clone()).collect();

    quote! (
        impl #ident {
            pub fn from_buf(packet_id: i32, buffer: &mut dyn ::falcon_core::network::buffer::PacketBufferRead) -> Result<Option<#ident>> {
                match packet_id {
                    #(#packet_ids => Ok(Some(#ident::#variant_idents(<#variant_types as ::falcon_core::network::packet::PacketDecode>::from_buf(buffer)?))),)*
                    _ => Ok(None),
                }
            }
        }
    ).into()
}
