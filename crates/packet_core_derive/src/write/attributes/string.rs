use proc_macro2::Span;
use syn::parse::Parse;

pub struct StringData {
    max_length: usize,
    span: Span,
}

impl StringData {
    pub fn span(&self) -> Span {
        self.span
    }
}

impl Parse for StringData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

impl PartialEq for StringData {
    fn eq(&self, other: &Self) -> bool {
        self.max_length == other.max_length
    }
}

impl Eq for StringData {}
