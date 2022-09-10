#![macro_use]

use falcon_proc_util::ErrorCatcher;
use proc_macro2::Span;
use syn::Error;

use self::{
    asref::{AsRefAttribute, AsRefKind},
    bytes::BytesAttribute,
    convert::{ConvertAttribute, FromAttribute, IntoAttribute},
    string::StringAttribute,
    varint::{VarI32Attribute, VarI64Attribute},
    vec::{ArrayAttribute, VecAttribute},
    PacketAttribute::*,
};

pub mod asref;
pub mod bytes;
pub mod convert;
pub mod string;
pub mod varint;
pub mod vec;

#[macro_use]
mod macros;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum PacketAttribute {
    Array(ArrayAttribute),
    AsRef(AsRefAttribute),
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
    pub fn check<'a, I>(&mut self, others: I) -> syn::Result<()>
    where
        I: Iterator<Item = &'a mut PacketAttribute>,
    {
        match self {
            String(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`string`").emit(),
            VarI32(_) | VarI64(_) => none_except!(
                Into(_) | From(_) | Convert(_),
                others,
                "`var32` and/or `var64`"
            )
            .emit(),
            Bytes(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`bytes`").emit(),
            Vec(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`vec`").emit(),
            Into(_) => all_except!(Convert(_), others, "`into`").emit(),
            From(_) => all_except!(Convert(_), others, "`from`").emit(),
            Convert(_) => all_except!(Into(_) | From(_), others, "`convert`").emit(),
            Array(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`array`").emit(),
            AsRef(ref mut data) => {
                let mut error = ErrorCatcher::new();
                others.for_each(|a| match a {
                    VarI32(_) | VarI64(_) | Vec(_) | Array(_) => {
                        error.add_error(Error::new(a.span(), "Incompatible with `as_ref`"))
                    }
                    String(_) => data.kind = AsRefKind::String,
                    Bytes(_) => data.kind = AsRefKind::Bytes,
                    _ => {}
                });
                error.emit()
            }
        }
    }

    pub fn is_outer(&self) -> bool {
        match self {
            String(_) => true,
            VarI32(_) => false,
            VarI64(_) => false,
            Bytes(_) => true,
            Vec(_) => true,
            Into(_) => false,
            From(_) => false,
            Convert(_) => false,
            Array(_) => false,
            AsRef(_) => false,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            String(data) => data.span(),
            VarI32(data) => data.span(),
            VarI64(data) => data.span(),
            Bytes(data) => data.span(),
            Vec(data) => data.span(),
            Into(data) => data.span(),
            From(data) => data.span(),
            Convert(data) => data.span(),
            Array(data) => data.span(),
            AsRef(data) => data.span(),
        }
    }
}

impl_parse! {
    Array = (ArrayAttribute as crate::kw::array),
    AsRef = (AsRefAttribute as crate::kw::asref),
    Bytes = (BytesAttribute as crate::kw::bytes),
    Convert = (ConvertAttribute as crate::kw::convert),
    Into = (IntoAttribute as crate::kw::into),
    From = (FromAttribute as crate::kw::from),
    String = (StringAttribute as crate::kw::string),
    VarI32 = (VarI32Attribute as crate::kw::var32),
    VarI64 = (VarI64Attribute as crate::kw::var64),
    Vec = (VecAttribute as crate::kw::vec),
}
