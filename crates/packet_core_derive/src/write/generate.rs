use syn::{parse_quote_spanned, spanned::Spanned, Expr};

use crate::attributes::{string::StringAttribute, PacketAttribute};

pub fn to_tokenstream(attribute: &PacketAttribute, field: Expr) -> Expr {
    match attribute {
        PacketAttribute::String(data) => generate_string(data, field),
        PacketAttribute::VarI32(_) => generate_var32(field),
        PacketAttribute::VarI64(_) => generate_var64(field),
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
