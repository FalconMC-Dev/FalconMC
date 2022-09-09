use syn::{parse_quote_spanned, spanned::Spanned, Expr, Type};

use crate::attributes::{convert::IntoAttribute, string::StringAttribute, PacketAttribute};

pub fn to_tokenstream(attribute: &PacketAttribute, field: Expr, field_ty: &Type) -> Expr {
    match attribute {
        PacketAttribute::String(data) => generate_string(data, field),
        PacketAttribute::VarI32(_) => generate_var32(field),
        PacketAttribute::VarI64(_) => generate_var64(field),
        PacketAttribute::Bytes(_) => generate_bytes(field),
        PacketAttribute::Vec(_) => generate_vec(field),
        PacketAttribute::Into(data) => generate_into(data, field, field_ty),
    }
}

fn generate_string(data: &StringAttribute, field: Expr) -> Expr {
    let len = &data.max_length;
    parse_quote_spanned! {field.span()=>
        ::falcon_packet_core::PacketWriteSeed::write(
            ::falcon_packet_core::PacketString::new(#len),
            #field,
            buffer,
        )
    }
}

fn generate_var32(field: Expr) -> Expr {
    parse_quote_spanned! {field.span()=>
        ::falcon_packet_core::VarI32::from(#field)
    }
}

fn generate_var64(field: Expr) -> Expr {
    parse_quote_spanned! {field.span()=>
        ::falcon_packet_core::VarI64::from(#field)
    }
}

fn generate_bytes(field: Expr) -> Expr {
    field
}

fn generate_vec(field: Expr) -> Expr {
    parse_quote_spanned! {field.span()=>
        ::falcon_packet_core::PacketWriteSeed::write(
            ::falcon_packet_core::PacketVec::new(0),
            #field,
            buffer,
        )
    }
}

fn generate_into(data: &IntoAttribute, field: Expr, field_ty: &Type) -> Expr {
    let target = &data.target;
    parse_quote_spanned! {field.span()=>
        <#field_ty as ::std::convert::Into<#target>>::into(#field)
    }
}
