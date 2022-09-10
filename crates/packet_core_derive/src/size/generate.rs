use syn::{Expr, Type, parse_quote_spanned, spanned::Spanned, Path};

use crate::attributes::{PacketAttribute, string::StringAttribute, asref::{AsRefAttribute, AsRefKind}};


pub fn to_tokenstream(attribute: &PacketAttribute, field: Expr, field_ty: &Type) -> Expr {
    match attribute {
        PacketAttribute::String(data) => generate_string(field),
        PacketAttribute::VarI32(_) => generate_var32(field),
        PacketAttribute::VarI64(_) => generate_var64(field),
        PacketAttribute::Vec(_) => generate_vec(field),
        PacketAttribute::Bytes(_) => passthrough(field),
        PacketAttribute::From(_) => passthrough(field),
        PacketAttribute::Into(data) => passthrough(field),
        PacketAttribute::Convert(data) => passthrough(field),
        PacketAttribute::Array(_) => generate_array(field),
        PacketAttribute::AsRef(data) => generate_asref(data, field),
    }
}

fn generate_array(field: Expr) -> Expr {
    parse_quote_spanned! {field.span()=>
        ::falcon_packet_core::PacketArray(&#field)
    }
}

fn generate_string(field: Expr) -> Expr {
    parse_quote_spanned! {field.span()=>
        ::falcon_packet_core::PacketSizeSeed::size(
            &::falcon_packet_core::PacketString::new(0),
            &#field,
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

fn passthrough(field: Expr) -> Expr {
    field
}

fn generate_vec(field: Expr) -> Expr {
    parse_quote_spanned! {field.span()=>
        ::falcon_packet_core::PacketSizeSeed::size(
            &::falcon_packet_core::PacketVec::new(0),
            &#field,
        )
    }
}

fn generate_asref(data: &AsRefAttribute, field: Expr) -> Expr {
    match data.kind {
        AsRefKind::Bytes => {
            parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::AsRefU8(&#field)
            }
        }
        AsRefKind::String => {
            parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::AsRefStr(&#field)
            }
        }
    }
}
