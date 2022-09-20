use derivative::Derivative;
use proc_macro2::Span;
use syn::{parse::Parse, spanned::Spanned, LitInt, Token};

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct StringAttribute {
    pub ident: kw::string,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub eq: Option<Token![=]>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub max_length: LitInt,
}

impl StringAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct ToStringAttribute {
    pub ident: kw::to_string,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub eq: Option<Token![=]>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub max_length: LitInt,
}

impl ToStringAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for StringAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::string>()?;
        let mut eq = None;
        let max_length = if input.peek(Token![=]) {
            eq = Some(input.parse::<Token![=]>()?);
            input.parse::<LitInt>()?
        } else {
            LitInt::new("32767", ident.span())
        };
        Ok(Self {
            ident,
            eq,
            max_length,
        })
    }
}

impl Parse for ToStringAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::to_string>()?;
        let mut eq = None;
        let max_length = if input.peek(Token![=]) {
            eq = Some(input.parse::<Token![=]>()?);
            input.parse::<LitInt>()?
        } else {
            LitInt::new("32767", ident.span())
        };
        Ok(Self {
            ident,
            eq,
            max_length,
        })
    }
}
