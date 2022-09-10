use falcon_proc_util::ErrorCatcher;
use proc_macro2::TokenStream;
use syn::{ItemStruct, Fields, Error, ItemImpl, Expr, parse_quote_spanned, spanned::Spanned};

use crate::util::ParsedFields;

use self::generate::to_tokenstream;

mod generate;

pub(crate) fn implement_size(item: ItemStruct) -> syn::Result<TokenStream> {
    let mut error = ErrorCatcher::new();

    match &item.fields {
        Fields::Named(fields) => {
            let fields = error.critical(ParsedFields::new(&fields.named))?;
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
    let mut writes: Vec<Expr> = Vec::with_capacity(parsed.fields.len());
    for (field, data) in parsed.fields {
        let ident = &field.ident;
        let field_ty = &field.ty;
        let mut field: Expr = parse_quote_spanned! {field.span()=> self.#ident };

        let mut different = false;
        for attribute in &data {
            different = attribute.is_outer();
            field = to_tokenstream(attribute, field, field_ty);
        }
        if !different {
            field = parse_quote_spanned! {field.span()=>
                ::falcon_packet_core::PacketSize::size(
                    #field,
                )
            }
        }
        writes.push(field);
    }

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    parse_quote_spanned! {item.ident.span()=>
        impl #impl_generics ::falcon_packet_core::PacketSize for #ident #ty_generics #where_clause {
            fn size(&self) -> usize {
                #(#writes)+*
            }
        }
    }
}
