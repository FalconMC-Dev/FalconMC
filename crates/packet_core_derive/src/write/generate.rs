use syn::{parse_quote_spanned, spanned::Spanned, Expr};

use crate::attributes::{string::StringAttribute, PacketAttribute};

pub fn to_tokenstream(attribute: &PacketAttribute, field: Expr) -> Expr {
    match attribute {
        PacketAttribute::String(data) => generate_string(data, field),
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
