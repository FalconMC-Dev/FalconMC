#![macro_use]

use falcon_proc_util::ErrorCatcher;
use proc_macro2::Span;
use syn::Error;

use self::{
    bytes::BytesAttribute,
    convert::IntoAttribute,
    string::StringAttribute,
    varint::{VarI32Attribute, VarI64Attribute},
    vec::VecAttribute,
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
            PacketAttribute::String(_) => except!(PacketAttribute::Into(_), others, "`string`"),
            PacketAttribute::VarI32(_) | PacketAttribute::VarI64(_) => {
                except!(PacketAttribute::Into(_), others, "`var32` and/or `var64`")
            }
            PacketAttribute::Bytes(_) => except!(PacketAttribute::Into(_), others, "`bytes`"),
            PacketAttribute::Vec(_) => except!(PacketAttribute::Into(_), others, "`vec`"),
            PacketAttribute::Into(_) => Ok(()),
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
        }
    }
}

impl_parse! {
    Bytes = (BytesAttribute as crate::kw::bytes),
    Into = (IntoAttribute as crate::kw::into),
    String = (StringAttribute as crate::kw::string),
    VarI32 = (VarI32Attribute as crate::kw::var32),
    VarI64 = (VarI64Attribute as crate::kw::var64),
    Vec = (VecAttribute as crate::kw::vec),
}
