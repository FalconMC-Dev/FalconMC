use derivative::Derivative;
use proc_macro2::Span;
use syn::{parse::Parse, LitStr, Path, Token};

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct IntoAttribute {
    pub ident: kw::into,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub eq: Token![=],
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub target: Path,
}

impl IntoAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for IntoAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::into>()?;
        let eq = input.parse::<Token![=]>()?;
        let target = input.parse::<LitStr>()?;
        Ok(Self {
            ident,
            eq,
            target: target.parse()?,
        })
    }
}
