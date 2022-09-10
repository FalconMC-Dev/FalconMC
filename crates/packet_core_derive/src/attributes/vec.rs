use derivative::Derivative;
use proc_macro2::Span;
use syn::{parse::Parse, Ident, Token};

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct ArrayAttribute {
    pub ident: kw::array,
}

impl ArrayAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for ArrayAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse::<kw::array>()?,
        })
    }
}

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct VecAttribute {
    pub ident: kw::vec,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub eq: Option<Token![=]>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub target: Option<Ident>,
}

impl VecAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for VecAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::vec>()?;
        if input.peek(Token![=]) {
            let eq = Some(input.parse::<Token![=]>()?);
            let target = Some(input.parse::<Ident>()?);
            Ok(Self { ident, eq, target })
        } else {
            Ok(Self {
                ident,
                eq: None,
                target: None,
            })
        }
    }
}
