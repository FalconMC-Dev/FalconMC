use proc_macro2::Span;
use syn::{parse::Parse, parse_quote_spanned, spanned::Spanned, LitInt, Token};

#[derive(Debug)]
pub struct StringData {
    max_length: usize,
    span: Span,
}

impl StringData {
    pub fn span(&self) -> Span {
        self.span
    }

    pub(crate) fn to_tokenstream(&self, field: syn::Expr) -> syn::Expr {
        let len = self.max_length;
        parse_quote_spanned! {field.span()=>
            ::falcon_packet_core::PacketWriteSeed::write(
                ::falcon_packet_core::PacketString::new(#len),
                #field,
                buffer,
            )
        }
    }
}

impl Parse for StringData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let size = input.parse::<LitInt>()?;
            let max_length = size.base10_parse()?;
            Ok(Self {
                max_length,
                span: size.span(),
            })
        } else {
            Ok(Self {
                max_length: 32767,
                span: input.span(),
            })
        }
    }
}

impl PartialEq for StringData {
    fn eq(&self, other: &Self) -> bool {
        self.max_length == other.max_length
    }
}

impl Eq for StringData {}
