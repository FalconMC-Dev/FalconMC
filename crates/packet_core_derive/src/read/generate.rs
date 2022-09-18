use proc_macro2::Span;
use syn::{parse_quote_spanned, spanned::Spanned, Expr, Type};

use crate::attributes::PacketAttribute::{self, *};

pub fn to_begin(attribute: &PacketAttribute, span: Span) -> Option<Expr> {
    match attribute {
        String(data) => {
            let len = &data.max_length;
            Some(parse_quote_spanned! {span=>
                ::falcon_packet_core::PacketReadSeed::read(
                    ::falcon_packet_core::PacketString::new(#len),
                    buffer,
                )?
            })
        }
        Vec(data) => {
            let target = &data.target;
            Some(parse_quote_spanned! {span=>
                ::falcon_packet_core::PacketReadSeed::read(
                    ::falcon_packet_core::PacketVec::new(#target.into()),
                    buffer,
                )?
            })
        }
        Array(_) => Some(parse_quote_spanned! {span=>
            ::falcon_packet_core::PacketReadSeed::read(
                ::falcon_packet_core::PacketArray::default(),
                buffer,
            )?
        }),
        Bytes(data) => Some(match data.target.as_ref() {
            Some(target) => parse_quote_spanned! {span=>
                ::falcon_packet_core::PacketReadSeed::read(
                    ::falcon_packet_core::Bytes::new(#target.into()),
                    buffer,
                )?
            },
            None => parse_quote_spanned! {span=>
                ::falcon_packet_core::PacketReadSeed::read(
                    ::falcon_packet_core::Bytes::new(buffer.remaining()),
                    buffer,
                )?
            },
        }),
        _ => None,
    }
}

pub fn to_tokenstream(attribute: &PacketAttribute, field: Expr, field_ty: &Type) -> Expr {
    match attribute {
        VarI32(_) => {
            parse_quote_spanned! {field.span()=>
                <#field_ty as ::std::convert::From<::falcon_packet_core::VarI32>>::from(#field)
            }
        }
        VarI64(_) => {
            parse_quote_spanned! {field.span()=>
                <#field_ty as ::std::convert::From<::falcon_packet_core::VarI64>>::from(#field)
            }
        }
        From(data) => {
            let target = &data.target;
            parse_quote_spanned! {field.span()=>
                <#field_ty as ::std::convert::From<#target>>::from(#field)
            }
        }
        Convert(data) => {
            let target = &data.target;
            parse_quote_spanned! {field.span()=>
                <#field_ty as ::std::convert::From<#target>>::from(#field)
            }
        }
        _ => field,
    }
}
