use derivative::Derivative;
use proc_macro2::Span;
use syn::{parse::Parse, Ident};

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct AsRefAttribute {
    pub ident: kw::asref,
    pub target: Option<Ident>,
}

impl AsRefAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for AsRefAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse::<kw::asref>()?,
            target: None,
        })
    }
}
