use derivative::Derivative;
use proc_macro2::Span;
use syn::{parse::Parse, Ident, Token};

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct VecAttribute {
    pub ident: kw::vec,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub eq: Token![=],
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub target: Ident,
}

impl VecAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for VecAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::vec>()?;
        let eq = input.parse::<Token![=]>()?;
        let target = input.parse::<Ident>()?;
        Ok(Self { ident, eq, target })
    }
}
