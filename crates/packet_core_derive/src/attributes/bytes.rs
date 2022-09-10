use derivative::Derivative;
use proc_macro2::Span;
use syn::{parse::Parse, Ident, Token};

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct BytesAttribute {
    pub ident: kw::bytes,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub eq: Option<Token![=]>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub target: Option<Ident>,
}

impl BytesAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for BytesAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::bytes>()?;
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
