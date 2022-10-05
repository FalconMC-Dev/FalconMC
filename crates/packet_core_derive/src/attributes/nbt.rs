use derivative::Derivative;
use proc_macro2::Span;
use syn::parse::Parse;

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct NBTAttribute {
    pub ident: kw::nbt,
}

impl NBTAttribute {
    pub fn span(&self) -> Span { self.ident.span }
}

impl Parse for NBTAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
        })
    }
}
