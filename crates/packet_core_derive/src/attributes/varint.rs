use derivative::Derivative;
use proc_macro2::Span;
use syn::parse::Parse;

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct VarI32Attribute {
    pub ident: kw::var32,
}

impl VarI32Attribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for VarI32Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::var32>()?;
        Ok(Self { ident })
    }
}

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct VarI64Attribute {
    pub ident: kw::var64,
}

impl VarI64Attribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for VarI64Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::var64>()?;
        Ok(Self { ident })
    }
}
