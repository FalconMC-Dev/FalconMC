use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::Item;

#[derive(Debug)]
pub struct ErrorCatcher {
    error: Option<syn::Error>,
}

impl ErrorCatcher {
    pub fn new() -> Self { Self { error: None } }

    pub fn add_error(&mut self, error: syn::Error) {
        match self.error {
            Some(ref mut err) => err.combine(error),
            None => self.error = Some(error),
        }
    }

    pub fn extend_error(&mut self, error: Result<(), syn::Error>) {
        match error {
            Ok(value) => value,
            Err(error) => self.add_error(error),
        }
    }

    pub fn critical<T>(&self, error: Result<T, syn::Error>) -> Result<T, syn::Error> {
        match error {
            Ok(value) => Ok(value),
            Err(mut error) => {
                if let Some(err) = &self.error {
                    error.combine(err.clone());
                }
                Err(error)
            },
        }
    }

    pub fn emit(self) -> syn::Result<()> {
        if let Some(err) = self.error {
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl Default for ErrorCatcher {
    fn default() -> Self { Self::new() }
}

pub struct ItemListing {
    pub content: Vec<Item>,
}

impl Parse for ItemListing {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { content: items })
    }
}

impl ToTokens for ItemListing {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for item in &self.content {
            item.to_tokens(tokens);
        }
    }
}
