use falcon_proc_util::ErrorCatcher;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{ItemStruct, Fields, Error};

pub(crate) fn implement_write(item: ItemStruct) -> syn::Result<TokenStream> {
    let mut error = ErrorCatcher::new();

    match item.fields {
        Fields::Named(fields) => {}
        _ => error.add_error(Error::new(item.fields.span(), "Only named fields are supported currently"))
    }

    error.emit()?;
    Ok(TokenStream::new())
}
