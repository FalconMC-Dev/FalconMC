#![macro_use]

use proc_macro2::Span;

use self::{
    bytes::BytesAttribute,
    convert::{ConvertAttribute, FromAttribute, IntoAttribute},
    link::LinkAttribute,
    nbt::NBTAttribute,
    string::StringAttribute,
    varint::{VarI32Attribute, VarI64Attribute},
    vec::{ArrayAttribute, VecAttribute},
    PacketAttribute::*,
};

pub mod bytes;
pub mod convert;
pub mod link;
pub mod nbt;
pub mod string;
pub mod varint;
pub mod vec;

#[macro_use]
pub mod macros;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum PacketAttribute {
    Array(ArrayAttribute),
    Bytes(BytesAttribute),
    Convert(ConvertAttribute),
    From(FromAttribute),
    Into(IntoAttribute),
    Link(LinkAttribute),
    Nbt(NBTAttribute),
    String(StringAttribute),
    VarI32(VarI32Attribute),
    VarI64(VarI64Attribute),
    Vec(VecAttribute),
}

impl PacketAttribute {
    pub fn span(&self) -> Span {
        match self {
            String(data) => data.span(),
            VarI32(data) => data.span(),
            VarI64(data) => data.span(),
            Bytes(data) => data.span(),
            Vec(data) => data.span(),
            Into(data) => data.span(),
            Link(data) => data.span(),
            From(data) => data.span(),
            Convert(data) => data.span(),
            Array(data) => data.span(),
            Nbt(data) => data.span(),
        }
    }
}

impl_parse! {
    Array = (ArrayAttribute as crate::kw::array),
    Bytes = (BytesAttribute as crate::kw::bytes),
    Convert = (ConvertAttribute as crate::kw::convert),
    Into = (IntoAttribute as crate::kw::into),
    From = (FromAttribute as crate::kw::from),
    Link = (LinkAttribute as crate::kw::link),
    Nbt = (NBTAttribute as crate::kw::nbt),
    String = (StringAttribute as crate::kw::string),
    VarI32 = (VarI32Attribute as crate::kw::var32),
    VarI64 = (VarI64Attribute as crate::kw::var64),
    Vec = (VecAttribute as crate::kw::vec),
}
