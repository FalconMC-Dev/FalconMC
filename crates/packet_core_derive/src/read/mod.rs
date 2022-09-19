use falcon_proc_util::ErrorCatcher;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_quote_spanned, spanned::Spanned, Error, Expr, Fields, ItemImpl, ItemStruct, Stmt};

use crate::util::ParsedFields;

use self::{
    check::validate,
    generate::{to_begin, to_tokenstream},
};

mod check;
mod generate;

pub(crate) fn implement_read(item: ItemStruct) -> syn::Result<TokenStream> {
    let mut error = ErrorCatcher::new();

    match &item.fields {
        Fields::Named(fields) => {
            let fields = error.critical(ParsedFields::new(&fields.named, validate))?;
            return Ok(generate_tokens(&item, fields).into_token_stream());
        }
        _ => error.add_error(Error::new(
            item.fields.span(),
            "Only named fields are supported currently",
        )),
    }

    error.emit()?;
    Ok(TokenStream::new())
}

fn generate_tokens(item: &ItemStruct, parsed: ParsedFields) -> ItemImpl {
    let mut temps: Vec<Stmt> = Vec::with_capacity(parsed.fields.len());
    let mut reads: Vec<Expr> = Vec::with_capacity(parsed.fields.len());

    for (field, data) in parsed.fields {
        let ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let should_skip;
        let mut tokens: Expr;
        if let Some(field) = data.first().and_then(|attr| to_begin(attr, field.span())) {
            should_skip = 1;
            tokens = field;
        } else {
            should_skip = 0;
            tokens = parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::PacketRead::read(
                    buffer,
                )?
            };
        }

        for attribute in data.iter().skip(should_skip) {
            tokens = to_tokenstream(attribute, tokens, field_ty);
        }

        temps.push(parse_quote_spanned! {tokens.span()=>
            let #ident: #field_ty = #tokens;
        });
        reads.push(parse_quote_spanned! {tokens.span()=>
            #ident
        })
    }

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    parse_quote_spanned! {item.ident.span()=>
        #[allow(clippy::useless_conversion)]
        #[automatically_derived]
        impl #impl_generics ::falcon_packet_core::PacketRead for #ident #ty_generics #where_clause {
            fn read<B>(buffer: &mut B) -> ::std::result::Result<Self, ::falcon_packet_core::ReadError>
            where
                B: ::bytes::Buf + ?Sized,
                Self: Sized
            {
                #(#temps)*
                Ok(#ident {
                    #(#reads),*
                })
            }
        }
    }
}
