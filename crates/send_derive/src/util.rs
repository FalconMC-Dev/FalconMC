use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Item;
use syn::parse::{Parse, ParseStream};

pub(crate) struct ItemListing {
    pub content: Vec<Item>,
}

impl Parse for ItemListing {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self {
            content: items,
        })
    }
}

impl ToTokens for ItemListing {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for item in &self.content {
            item.to_tokens(tokens);
        }
    }
}
