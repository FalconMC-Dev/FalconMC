use derivative::Derivative;
use proc_macro2::Span;
use syn::parse::Parse;

use crate::kw;

#[derive(Debug)]
pub enum AsRefKind {
    Bytes,
    String,
}

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct AsRefAttribute {
    pub ident: kw::asref,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub kind: AsRefKind,
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
            kind: AsRefKind::Bytes,
        })
    }
}
