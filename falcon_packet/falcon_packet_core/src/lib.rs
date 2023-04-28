#![doc = include_str!("../README.md")]

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;

mod data;
pub(crate) mod kw;
mod processing;
mod tests;
mod traits;

pub use data::*;
pub use processing::doctest_impls;
use syn::{parse2, parse_quote, Attribute, FnArg, Ident, Stmt, Visibility};
pub use traits::{packet_read, packet_size, packet_write};

use crate::processing::{field_to_read, field_to_size, field_to_write};

pub fn packet_core(input: TokenStream) -> TokenStream {
    if let Ok(valid) = parse2::<ValiditySyntax>(input.clone()) {
        if valid.has_packet {
            match parse2::<PacketSyntax>(input) {
                Ok(syntax) => {
                    let structdef = gen_struct(
                        &syntax.packet_name,
                        &syntax.attrs,
                        &syntax.vis,
                        &syntax.init,
                        syntax.fields.iter().map(|f| &f.struct_field),
                        syntax.inputs.iter(),
                    );
                    let sizeimpl = gen_size(&syntax);
                    let readimpl = gen_read(&syntax);
                    let writeimpl = gen_write(&syntax);
                    quote!(#structdef #sizeimpl #readimpl #writeimpl)
                },
                Err(err) => err.to_compile_error(),
            }
        } else {
            match parse2::<StructSyntax>(input) {
                Ok(syntax) => {
                    let structdef = gen_struct(
                        &syntax.packet_name,
                        &syntax.attrs,
                        &syntax.vis,
                        &syntax.init,
                        &syntax.fields,
                        syntax.inputs.iter(),
                    );
                    quote!(#structdef)
                },
                Err(err) => err.to_compile_error(),
            }
        }
    } else {
        abort! { input, "Unsupported syntax" }
    }
}

pub fn gen_struct<'a, F, A>(
    packet_name: &Ident,
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    init: &Vec<Stmt>,
    fields: F,
    inputs: A,
) -> TokenStream
where
    F: IntoIterator<Item = &'a PacketStructField> + Clone,
    A: Iterator<Item = &'a FnArg>,
{
    let field_defs = fields.clone().into_iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote!(#ident: #ty)
    });
    let structdef = quote! {
        #(#attrs)*
        #vis struct #packet_name {
            #(#field_defs),*
        }
    };

    let field_args = fields.clone().into_iter().filter_map(|f| {
        if f.let_token.is_none() {
            let ident = &f.ident;
            let ty = &f.ty;
            Some(quote!(#ident: #ty))
        } else {
            None
        }
    });
    let field_inits = fields.clone().into_iter().filter_map(|f| {
        if f.let_token.is_some() {
            let ident = &f.ident;
            let ty = &f.ty;
            let init = &f.init;
            Some(quote!(let #ident: #ty = #init;))
        } else {
            None
        }
    });
    let fields = fields.into_iter().map(|f| &f.ident);

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
        field_to_size(&f.struct_field.ty, &f.spec, ident)
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
    let buffer = parse_quote!(buffer);
    let field_impls = input
        .fields
        .iter()
        .map(|f| field_to_read(&f.struct_field.ty, &f.spec, &f.struct_field.ident, &buffer));
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
    let buffer = parse_quote!(buffer);
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
        field_to_write(&f.struct_field.ty, &f.spec, ident, &buffer)
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