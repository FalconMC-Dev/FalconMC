use falcon_proc_util::ErrorCatcher;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Error, Expr, Fields, ItemImpl, ItemStruct, Stmt};

use self::check::{get_replaced, validate};
use self::generate::{to_end, to_preprocess, to_tokenstream};
use crate::util::ParsedFields;

mod check;
mod generate;

pub(crate) fn implement_size(item: ItemStruct) -> syn::Result<TokenStream> {
    let mut error = ErrorCatcher::new();

    match &item.fields {
        Fields::Named(fields) => {
            let fields = error.critical(ParsedFields::new(&fields.named, validate))?;
            return Ok(generate_tokens(&item, fields).into_token_stream());
        },
        _ => error.add_error(Error::new(item.fields.span(), "Only named fields are supported currently")),
    }

    error.emit()?;
    Ok(TokenStream::new())
}

fn generate_tokens(item: &ItemStruct, parsed: ParsedFields) -> ItemImpl {
    let mut preprocess: Vec<Stmt> = Vec::new();
    let mut writes: Vec<Expr> = Vec::with_capacity(parsed.fields.len());

    let replace = get_replaced(&parsed.fields);

    for (field, data) in parsed.fields {
        let ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let mut field: Expr = if replace.contains(ident) {
            parse_quote_spanned! {field.span()=> <#field_ty as ::std::convert::From<usize>>::from(#ident)}
        } else {
            parse_quote_spanned! {field.span()=> self.#ident}
        };

        let mut end = None;

        for (i, attribute) in data.iter().enumerate() {
            field = to_tokenstream(attribute, field, field_ty);
            if i == data.len() - 1 {
                if let Some(process) = to_preprocess(attribute, field.clone()) {
                    preprocess.extend(process);
                }
                end = to_end(attribute, field.clone());
            }
        }

        writes.push(end.unwrap_or_else(|| {
            parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::PacketSize::size(
                    &#field,
                )
            }
        }));
    }

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    parse_quote_spanned! {item.ident.span()=>
        #[allow(clippy::useless_conversion)]
        #[automatically_derived]
        impl #impl_generics ::falcon_packet_core::PacketSize for #ident #ty_generics #where_clause {
            fn size(&self) -> usize {
                #(#preprocess)*
                #(#writes)+*
            }
        }
    }
}
