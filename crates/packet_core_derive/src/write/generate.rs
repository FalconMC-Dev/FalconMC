use quote::format_ident;
use syn::{parse_quote_spanned, spanned::Spanned, Expr, Stmt, Type};

use crate::attributes::PacketAttribute::{self, *};

pub fn to_preprocess(attribute: &PacketAttribute, field: Expr) -> Option<Stmt> {
    match attribute {
        Vec(data) => {
            let target = &data.target;
            Some(parse_quote_spanned! {field.span()=>
                self.#target = #field.len().into();
            })
        }
        Bytes(data) => data.target.as_ref().map(|target| {
            parse_quote_spanned! {field.span()=>
                self.#target = ::falcon_packet_core::PacketSizeSeed::size(
                    &::falcon_packet_core::AsRefU8::default(),
                    &#field,
                ).into();
            }
        }),
        Link(data) => {
            let prefix = format_ident!("{}_value", data.prefix);
            let target = &data.target;
            let others = data.others.as_ref();
            Some(match others.map(|o| o.into_iter()) {
                Some(others) => parse_quote_spanned! {field.span()=>
                    self.#target = #prefix(&#field, #(&self.#others),*).into();
                },
                None => parse_quote_spanned! {field.span()=>
                    self.#target = #prefix(&#field).into();
                },
            })
        }
        _ => None,
    }
}

pub fn to_end(attribute: &PacketAttribute, field: Expr) -> Option<Stmt> {
    match attribute {
        String(data) => {
            let len = &data.max_length;
            Some(parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::PacketWriteSeed::write(
                    ::falcon_packet_core::PacketString::new(#len),
                    #field,
                    buffer,
                )?;
            })
        }
        ToString(data) => {
            let len = &data.max_length;
            Some(parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::PacketWriteSeed::write(
                    ::falcon_packet_core::PacketString::new(#len),
                    ::std::string::ToString::to_string(&#field),
                    buffer,
                )?;
            })
        }
        Vec(_) => Some(parse_quote_spanned! {field.span()=>
            ::falcon_packet_core::PacketWriteSeed::write(
                ::falcon_packet_core::PacketVec::new(0),
                #field,
                buffer,
            )?;
        }),
        Array(_) => Some(parse_quote_spanned! {field.span()=>
            ::falcon_packet_core::PacketWriteSeed::write(
                ::falcon_packet_core::PacketArray::default(),
                #field,
                buffer,
            )?;
        }),
        Bytes(_) => Some(parse_quote_spanned! {field.span()=>
            ::falcon_packet_core::PacketWriteSeed::write(
                ::falcon_packet_core::AsRefU8::default(),
                #field,
                buffer,
            )?;
        }),
        Link(data) => {
            let prefix = format_ident!("{}_write", data.prefix);
            Some(parse_quote_spanned! {field.span()=>
                #prefix(#field, buffer)?;
            })
        }
        Nbt(_) => Some(parse_quote_spanned! {field.span()=>
            {
                let writer = ::falcon_packet_core::special::Writer::new(buffer);
                ::fastnbt::to_writer(writer, &#field)?;
            }
        }),
        _ => None,
    }
}

pub fn to_tokenstream(attribute: &PacketAttribute, field: Expr, field_ty: &Type) -> Expr {
    match attribute {
        VarI32(_) => {
            parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::VarI32::from(#field)
            }
        }
        VarI64(_) => {
            parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::VarI64::from(#field)
            }
        }
        Into(data) => {
            let target = &data.target;
            parse_quote_spanned! {field.span()=>
                <#field_ty as ::std::convert::Into<#target>>::into(#field)
            }
        }
        Convert(data) => {
            let target = &data.target;
            parse_quote_spanned! {field.span()=>
                <#field_ty as ::std::convert::Into<#target>>::into(#field)
            }
        }
        _ => field,
    }
}
