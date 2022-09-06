#![macro_use]

use proc_macro2::Span;

use self::string::StringAttribute;

pub mod string;
#[macro_use]
mod macros;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum PacketAttribute {
    String(StringAttribute),
}

impl PacketAttribute {
    pub fn check<'a, I>(&self, _others: I) -> syn::Result<()>
    where
        I: Iterator<Item = &'a PacketAttribute>,
    {
        Ok(())
    }

    pub fn is_outer(&self) -> bool {
        match self {
            PacketAttribute::String(_) => true,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            PacketAttribute::String(data) => data.span(),
        }
    }
}

impl_parse! {
    String = (StringAttribute as crate::kw::string),
}
