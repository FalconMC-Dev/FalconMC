#![macro_use]

use falcon_proc_util::ErrorCatcher;
use proc_macro2::Span;
use syn::Error;

use self::{
    bytes::BytesAttribute,
    convert::{ConvertAttribute, FromAttribute, IntoAttribute},
    string::StringAttribute,
    varint::{VarI32Attribute, VarI64Attribute},
    vec::VecAttribute,
    PacketAttribute::*,
};

pub mod bytes;
pub mod convert;
pub mod string;
pub mod varint;
pub mod vec;

#[macro_use]
mod macros;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum PacketAttribute {
    Bytes(BytesAttribute),
    Convert(ConvertAttribute),
    From(FromAttribute),
    Into(IntoAttribute),
    String(StringAttribute),
    VarI32(VarI32Attribute),
    VarI64(VarI64Attribute),
    Vec(VecAttribute),
}

impl PacketAttribute {
    pub fn check<'a, I>(&self, others: I) -> syn::Result<()>
    where
        I: Iterator<Item = &'a PacketAttribute>,
    {
        match self {
            String(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`string`"),
            VarI32(_) | VarI64(_) => none_except!(
                Into(_) | From(_) | Convert(_),
                others,
                "`var32` and/or `var64`"
            ),
            Bytes(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`bytes`"),
            Vec(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`vec`"),
            Into(_) => all_except!(Convert(_), others, "`into`"),
            From(_) => all_except!(Convert(_), others, "`from`"),
            Convert(_) => all_except!(Into(_) | From(_), others, "`convert`"),
        }
    }

    pub fn is_outer(&self) -> bool {
        match self {
            PacketAttribute::String(_) => true,
            PacketAttribute::VarI32(_) => false,
            PacketAttribute::VarI64(_) => false,
            PacketAttribute::Bytes(_) => true,
            PacketAttribute::Vec(_) => true,
            PacketAttribute::Into(_) => false,
            PacketAttribute::From(_) => false,
            PacketAttribute::Convert(_) => false,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            PacketAttribute::String(data) => data.span(),
            PacketAttribute::VarI32(data) => data.span(),
            PacketAttribute::VarI64(data) => data.span(),
            PacketAttribute::Bytes(data) => data.span(),
            PacketAttribute::Vec(data) => data.span(),
            PacketAttribute::Into(data) => data.span(),
            PacketAttribute::From(data) => data.span(),
            PacketAttribute::Convert(data) => data.span(),
        }
    }
}

impl_parse! {
    Bytes = (BytesAttribute as crate::kw::bytes),
    Convert = (ConvertAttribute as crate::kw::convert),
    Into = (IntoAttribute as crate::kw::into),
    From = (FromAttribute as crate::kw::from),
    String = (StringAttribute as crate::kw::string),
    VarI32 = (VarI32Attribute as crate::kw::var32),
    VarI64 = (VarI64Attribute as crate::kw::var64),
    Vec = (VecAttribute as crate::kw::vec),
}
