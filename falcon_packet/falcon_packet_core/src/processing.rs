use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, parse_str, Expr, Stmt};

use crate::{FieldSpec, PacketField};

pub fn doctest_impls(syntax: &str, read: &str, write: &str, size: &str) {
    let field = parse_str::<PacketField>(syntax).expect("Invalid Packet field syntax");
    let ident = &field.struct_field.ident;
    let read = parse_str::<Stmt>(read).expect("Read couldn't be parsed as tokens");
    let read_expected = parse2::<Stmt>(field_to_read(&field)).unwrap();
    let write = parse_str::<Stmt>(write).expect("Write couldn't be parsed as tokens");
    let write_expected = parse2::<Stmt>(field_to_write(&field, quote!(self.#ident))).unwrap();
    let size = parse_str::<Expr>(size).expect("Size couldn't be parsed as tokens");
    let size_expected = parse2::<Expr>(field_to_size(&field, quote!(self.#ident))).unwrap();
    assert_eq!(quote!(#read_expected).to_string(), quote!(#read).to_string(), "read failed");
    assert_eq!(quote!(#write_expected).to_string(), quote!(#write).to_string(), "write failed");
    assert_eq!(quote!(#size_expected).to_string(), quote!(#size).to_string(), "size failed");
}

pub fn field_to_size(field: &PacketField, ident: TokenStream) -> TokenStream {
    let ty = &field.struct_field.ty;
    match field.spec {
        FieldSpec::Direct => quote!(::falcon_packet::PacketSize::size(&#ident)),
        FieldSpec::DirectAs(ref ty) => quote!(::falcon_packet::PacketSize::size(&(#ident as #ty))),
        FieldSpec::Var32 => {
            quote!(::falcon_packet::PacketSize::size(&<#ty as Into<::falcon_packet::primitives::VarI32>>::into(#ident)))
        },
        FieldSpec::Var64 => {
            quote!(::falcon_packet::PacketSize::size(&<#ty as Into<::falcon_packet::primitives::VarI64>>::into(#ident)))
        },
        FieldSpec::String(_) => quote!(::falcon_packet::PacketSize::size(<#ty as AsRef<str>>::as_ref(&#ident))),
        FieldSpec::Bytes(_) => quote!(::falcon_packet::PacketSize::size(&#ident)),
        FieldSpec::Rest => quote!(::falcon_packet::PacketSize::size(&#ident)),
        FieldSpec::Array => quote!(::falcon_packet::PacketSize::size(&#ident)),
        FieldSpec::ByteArray => quote!(::falcon_packet::PacketSize::size(&#ident)),
        FieldSpec::Nbt => quote!(::falcon_packet::primitives::nbt_size(&#ident)),
    }
}

pub fn field_to_read(field: &PacketField) -> TokenStream {
    let ident = &field.struct_field.ident;
    let ty = &field.struct_field.ty;
    match field.spec {
        FieldSpec::Direct => quote!(let #ident = ::falcon_packet::PacketRead::read(buffer)?;),
        FieldSpec::DirectAs(ref ty2) => {
            quote!(let #ident = <#ty2 as ::falcon_packet::PacketRead>::read(buffer)? as #ty;)
        },
        FieldSpec::Var32 => {
            quote!(let #ident = <::falcon_packet::primitives::VarI32 as ::falcon_packet::PacketRead>::read(buffer)?.into();)
        },
        FieldSpec::Var64 => {
            quote!(let #ident = <::falcon_packet::primitives::VarI64 as ::falcon_packet::PacketRead>::read(buffer)?.into();)
        },
        FieldSpec::String(max_len) => quote!(let #ident = ::falcon_packet::PacketReadSeed::read(#max_len, buffer)?;),
        FieldSpec::Bytes((ref field, _)) => {
            quote!(let #ident = ::falcon_packet::PacketReadSeed::read(#field as usize, buffer)?;)
        },
        FieldSpec::Rest => quote!(let #ident = ::falcon_packet::PacketReadSeed::read((), buffer)?;),
        FieldSpec::Array => quote!(let #ident = ::falcon_packet::primitives::array_read(buffer)?;),
        FieldSpec::ByteArray => quote!(let #ident = ::falcon_packet::primitives::bytearray_read(buffer)?;),
        FieldSpec::Nbt => quote!(let #ident = ::falcon_packet::primitives::nbt_read(buffer)?;),
    }
}

pub fn field_to_write(field: &PacketField, ident: TokenStream) -> TokenStream {
    let ty = &field.struct_field.ty;
    match field.spec {
        FieldSpec::Direct => quote!(::falcon_packet::PacketWrite::write(&#ident, buffer)?;),
        FieldSpec::DirectAs(ref ty) => quote!(::falcon_packet::PacketWrite::write(&(#ident as #ty), buffer)?;),
        FieldSpec::Var32 => {
            quote!(::falcon_packet::PacketWrite::write(&<#ty as Into<::falcon_packet::primitives::VarI32>>::into(#ident), buffer)?;)
        },
        FieldSpec::Var64 => {
            quote!(::falcon_packet::PacketWrite::write(&<#ty as Into<::falcon_packet::primitives::VarI64>>::into(#ident), buffer)?;)
        },
        FieldSpec::String(max_len) => {
            quote!(::falcon_packet::PacketWriteSeed::write(#max_len, <#ty as AsRef<str>>::as_ref(&#ident), buffer)?;)
        },
        FieldSpec::Bytes(_) => {
            quote!(::falcon_packet::PacketWrite::write(<#ty as AsRef<[u8]>>::as_ref(&#ident), buffer)?;)
        },
        FieldSpec::Rest => quote!(::falcon_packet::PacketWrite::write(<#ty as AsRef<[u8]>>::as_ref(&#ident), buffer)?;),
        FieldSpec::Array => quote!(::falcon_packet::PacketWrite::write(&#ident, buffer)?;),
        FieldSpec::ByteArray => quote!(::falcon_packet::PacketWrite::write(&#ident, buffer)?;),
        FieldSpec::Nbt => quote!(::falcon_packet::primitives::nbt_write(&#ident, buffer)?;),
    }
}
