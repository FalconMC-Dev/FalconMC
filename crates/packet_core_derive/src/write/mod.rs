use falcon_proc_util::ErrorCatcher;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{Error, Fields, ItemStruct};

use crate::util::ParsedFields;

use attributes::WriteAttributes;

mod attributes;

pub(crate) fn implement_write(item: ItemStruct) -> syn::Result<TokenStream> {
    let mut error = ErrorCatcher::new();

    match item.fields {
        Fields::Named(fields) => {
            let _fields: ParsedFields<WriteAttributes> = error.critical(ParsedFields::new(&fields.named))?;
        }
        _ => error.add_error(
            Error::new(
                item.fields.span(),
                "Only named fields are supported currently",
            )
        ),
    }

    error.emit()?;
    Ok(TokenStream::new())
}
