use derivative::Derivative;
use proc_macro2::Span;
use syn::{parse::Parse, punctuated::Punctuated, Ident, LitStr, Token};

use crate::kw;

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq)]
pub struct LinkAttribute {
    pub ident: kw::link,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub eq: Token![=],
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub target: Ident,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub comma: Option<Token![,]>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub others: Option<Punctuated<Ident, Token![,]>>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub with: kw::with,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub prefix: Ident,
}

impl LinkAttribute {
    pub fn span(&self) -> Span {
        self.ident.span
    }
}

impl Parse for LinkAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<kw::link>()?;
        let eq = input.parse()?;
        let fields = input.parse::<LitStr>()?.parse::<LinkInner>()?;
        Ok(Self {
            ident,
            eq,
            target: fields.target,
            comma: fields.comma,
            others: fields.others,
            with: fields.with,
            prefix: fields.prefix,
        })
    }
}

struct LinkInner {
    target: Ident,
    comma: Option<Token![,]>,
    others: Option<Punctuated<Ident, Token![,]>>,
    with: kw::with,
    prefix: Ident,
}

impl Parse for LinkInner {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target = input.parse()?;
        let (comma, others) = if input.peek(Token![,]) {
            let comma = input.parse()?;
            let others = Punctuated::<Ident, Token![,]>::parse_separated_nonempty(input)?;
            (Some(comma), Some(others))
        } else {
            (None, None)
        };
        let with = input.parse()?;
        let prefix = input.parse()?;
        Ok(Self {
            target,
            comma,
            others,
            with,
            prefix,
        })
    }
}
