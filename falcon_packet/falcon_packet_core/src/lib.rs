#![doc = include_str!("../README.md")]

use proc_macro2::TokenStream;
use quote::quote;

mod data;
pub(crate) mod kw;
mod tests;

pub use data::*;
use syn::{parse2, Ident};

pub fn packet_core(input: TokenStream) -> TokenStream {
    let packet_syntax = match parse2::<PacketSyntax>(input) {
        Ok(syntax) => syntax,
        Err(error) => return error.to_compile_error(),
    };

    let structdef = gen_struct(&packet_syntax);
    let sizeimpl = gen_size(&packet_syntax);
    let readimpl = gen_read(&packet_syntax);
    let writeimpl = gen_write(&packet_syntax);
    quote!(#structdef #sizeimpl #readimpl #writeimpl)
}

pub fn gen_struct(input: &PacketSyntax) -> TokenStream {
    let attrs = &input.attrs;
    let vis = &input.vis;
    let packet_name = &input.packet_name;
    let field_defs = input.fields.iter().map(|f| {
        let ident = &f.struct_field.ident;
        let ty = &f.struct_field.ty;
        quote!(#ident: #ty)
    });
    let structdef = quote! {
        #(#attrs)*
        #vis struct #packet_name {
            #(#field_defs),*
        }
    };

    let inputs = input.inputs.iter();
    let init = &input.init;
    let field_args = input.fields.iter().map(|f| &f.struct_field).filter_map(|f| {
        if f.let_token.is_none() {
            let ident = &f.ident;
            let ty = &f.ty;
            Some(quote!(#ident: #ty))
        } else {
            None
        }
    });
    let field_inits = input.fields.iter().map(|f| &f.struct_field).filter_map(|f| {
        if f.let_token.is_some() {
            let ident = &f.ident;
            let ty = &f.ty;
            let init = &f.init;
            Some(quote!(let #ident: #ty = #init;))
        } else {
            None
        }
    });
    let fields = input.fields.iter().map(|f| &f.struct_field.ident);

    let structinit = quote! {
        impl #packet_name {
            pub fn new(#(#inputs,)* #(#field_args),*) -> Self {
                #(#init)*
                #(#field_inits)*
                Self {
                    #(#fields),*
                }
            }
        }
    };
    quote!(#structdef #structinit)
}

pub fn gen_size(input: &PacketSyntax) -> TokenStream {
    let packet_name = &input.packet_name;
    let field_overwrites: Vec<&Ident> = input
        .fields
        .iter()
        .filter_map(|f| match f.spec {
            FieldSpec::Bytes((ref ident, _)) => Some(ident),
            _ => None,
        })
        .collect();
    let fields_preprocess = input.fields.iter().filter_map(|f| match f.spec {
        FieldSpec::Bytes((ref ident, ref expr)) => Some(quote!(let #ident = #expr;)),
        _ => None,
    });
    let field_impls = input.fields.iter().map(|f| {
        let ident = &f.struct_field.ident;
        let ident = if field_overwrites.contains(&ident) {
            quote!(#ident)
        } else {
            quote!(self.#ident)
        };
        let ty = &f.struct_field.ty;
        match f.spec {
            FieldSpec::Direct => quote!(::falcon_packet::PacketSize::size(&#ident)),
            FieldSpec::DirectAs(ref ty) => quote!(::falcon_packet::PacketSize::size(&(#ident as #ty))),
            FieldSpec::Var32 => quote!(::falcon_packet::PacketSize::size(&<#ty as Into<::falcon_packet::primitives::VarI32>>::into(#ident))),
            FieldSpec::Var64 => quote!(::falcon_packet::PacketSize::size(&<#ty as Into<::falcon_packet::primitives::VarI64>>::into(#ident))),
            FieldSpec::String(_) => quote!(::falcon_packet::PacketSize::size(<#ty as AsRef<str>>::as_ref(&#ident))),
            FieldSpec::Bytes(_) => quote!(::falcon_packet::PacketSize::size(&#ident)),
            FieldSpec::Rest => quote!(::falcon_packet::PacketSize::size(&#ident)),
            FieldSpec::Array => quote!(::falcon_packet::PacketSize::size(&#ident)),
            FieldSpec::ByteArray => quote!(::falcon_packet::PacketSize::size(&#ident)),
            FieldSpec::Nbt => quote!(::falcon_packet::primitives::nbt_size(&#ident)),
        }
    });

    quote!(
        impl ::falcon_packet::PacketSize for #packet_name {
            fn size(&self) -> usize {
                #(#fields_preprocess)*
                #(#field_impls)+*
            }
        }
    )
}

pub fn gen_read(input: &PacketSyntax) -> TokenStream {
    let packet_name = &input.packet_name;
    let field_impls = input.fields.iter().map(|f| {
        let ident = &f.struct_field.ident;
        let ty = &f.struct_field.ty;
        match f.spec {
            FieldSpec::Direct => quote!(let #ident = ::falcon_packet::PacketRead::read(buffer)?;),
            FieldSpec::DirectAs(ref ty2) => quote!(let #ident = <#ty2 as ::falcon_packet::PacketRead>::read(buffer)? as #ty;),
            FieldSpec::Var32 => quote!(let #ident = <::falcon_packet::primitives::VarI32 as ::falcon_packet::PacketRead>::read(buffer)?.into();),
            FieldSpec::Var64 => quote!(let #ident = <::falcon_packet::primitives::VarI64 as ::falcon_packet::PacketRead>::read(buffer)?.into();),
            FieldSpec::String(max_len) => quote!(let #ident = ::falcon_packet::PacketReadSeed::read(#max_len, buffer)?;),
            FieldSpec::Bytes((ref field, _)) => quote!(let #ident = ::falcon_packet::PacketReadSeed::read(self.#field as usize, buffer)?;),
            FieldSpec::Rest => quote!(let #ident = ::falcon_packet::PacketReadSeed::read((), buffer)?;),
            FieldSpec::Array => quote!(let #ident = ::falcon_packet::primitives::array_read(buffer)?;),
            FieldSpec::ByteArray => quote!(let #ident = ::falcon_packet::primitives::bytearray_read(buffer)?;),
            FieldSpec::Nbt => quote!(let #ident = ::falcon_packet::primitives::nbt_read(buffer)?;),
        }
    });
    let fields = input.fields.iter().map(|f| &f.struct_field.ident);

    quote!(
        impl ::falcon_packet::PacketRead for #packet_name {
            fn read<B>(buffer: &mut B) -> ::std::result::Result<Self, ::falcon_packet::ReadError>
            where
                B: ::bytes::Buf + ?Sized,
                Self: Sized
            {
                #(#field_impls)*
                Ok(Self { #(#fields),* })
            }
        }
    )
}

pub fn gen_write(input: &PacketSyntax) -> TokenStream {
    let packet_name = &input.packet_name;
    let field_overwrites: Vec<&Ident> = input
        .fields
        .iter()
        .filter_map(|f| match f.spec {
            FieldSpec::Bytes((ref ident, _)) => Some(ident),
            _ => None,
        })
        .collect();
    let fields_preprocess = input.fields.iter().filter_map(|f| match f.spec {
        FieldSpec::Bytes((ref ident, ref expr)) => Some(quote!(let #ident = #expr;)),
        _ => None,
    });
    let fields_impl = input.fields.iter().map(|f| {
        let ident = &f.struct_field.ident;
        let ident = if field_overwrites.contains(&ident) {
            quote!(#ident)
        } else {
            quote!(self.#ident)
        };
        let ty = &f.struct_field.ty;
        match f.spec {
            FieldSpec::Direct => quote!(::falcon_packet::PacketWrite::write(&#ident, buffer)?;),
            FieldSpec::DirectAs(ref ty) => quote!(::falcon_packet::PacketWrite::write(&(#ident as #ty), buffer)?;),
            FieldSpec::Var32 => quote!(::falcon_packet::PacketWrite::write(&<#ty as Into<::falcon_packet::primitives::VarI32>>::into(#ident), buffer)?;),
            FieldSpec::Var64 => quote!(::falcon_packet::PacketWrite::write(&<#ty as Into<::falcon_packet::primitives::VarI64>>::into(#ident), buffer)?;),
            FieldSpec::String(max_len) => quote!(::falcon_packet::PacketWriteSeed::write(#max_len, <#ty as AsRef<str>>::as_ref(&#ident), buffer)?;),
            FieldSpec::Bytes(_) => quote!(::falcon_packet::PacketWrite::write(<#ty as AsRef<[u8]>>::as_ref(&#ident), buffer)?;),
            FieldSpec::Rest => quote!(::falcon_packet::PacketWrite::write(<#ty as AsRef<[u8]>>::as_ref(&#ident), buffer)?;),
            FieldSpec::Array => quote!(::falcon_packet::PacketWrite::write(&#ident, buffer)?;),
            FieldSpec::ByteArray => quote!(::falcon_packet::PacketWrite::write(&#ident, buffer)?;),
            FieldSpec::Nbt => quote!(::falcon_packet::primitives::nbt_write(&#ident, buffer)?;),
        }
    });

    quote!(
        impl ::falcon_packet::PacketWrite for #packet_name {
            fn write<B>(&self, buffer: &mut B) -> ::std::result::Result<(), ::falcon_packet::WriteError>
            where
                B: ::bytes::BufMut
            {
                #(#fields_preprocess)*
                #(#fields_impl)*
                Ok(())
            }
        }
    )
}
