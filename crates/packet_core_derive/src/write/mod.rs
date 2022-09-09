use falcon_proc_util::ErrorCatcher;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Error, Expr, Fields, ItemImpl, ItemStruct, Stmt};

use crate::util::ParsedFields;

use self::generate::to_tokenstream;

mod generate;

pub(crate) fn implement_write(item: ItemStruct) -> syn::Result<TokenStream> {
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
    let mut writes: Vec<Stmt> = Vec::with_capacity(parsed.fields.len());
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
                ::falcon_packet_core::PacketWrite::write(
                    #field,
                    buffer,
                )
            }
        }
        writes.push(parse_quote_spanned! {field.span()=> #field?; })
    }

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    parse_quote_spanned! {item.ident.span()=>
        impl #impl_generics ::falcon_packet_core::PacketWrite for #ident #ty_generics #where_clause {
            fn write<B>(self, buffer: &mut B) -> ::std::result::Result<(), ::falcon_packet_core::WriteError>
            where
                B: ::bytes::BufMut + ?Sized
            {
                #(#writes)*
                Ok(())
            }
        }
    }
}
