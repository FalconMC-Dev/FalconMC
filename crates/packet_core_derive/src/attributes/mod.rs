#![macro_use]

use falcon_proc_util::ErrorCatcher;
use proc_macro2::Span;
use syn::Error;

use self::{
    string::StringAttribute,
    varint::{VarI32Attribute, VarI64Attribute},
};

pub mod string;
pub mod varint;

#[macro_use]
mod macros;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum PacketAttribute {
    String(StringAttribute),
    VarI32(VarI32Attribute),
    VarI64(VarI64Attribute),
}

impl PacketAttribute {
    pub fn check<'a, I>(&self, others: I) -> syn::Result<()>
    where
        I: Iterator<Item = &'a PacketAttribute>,
    {
        match self {
            PacketAttribute::String(_) => Ok(()),
            PacketAttribute::VarI32(_) | PacketAttribute::VarI64(_) => {
                let mut error = ErrorCatcher::new();
                others.for_each(|a| {
                    error.add_error(Error::new(
                        a.span(),
                        "Incompatible with `var32` and/or `var64`.",
                    ))
                });
                error.emit()
            }
        }
    }

    pub fn is_outer(&self) -> bool {
        match self {
            PacketAttribute::String(_) => true,
            PacketAttribute::VarI32(_) => false,
            PacketAttribute::VarI64(_) => false,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            PacketAttribute::String(data) => data.span(),
            PacketAttribute::VarI32(data) => data.span(),
            PacketAttribute::VarI64(data) => data.span(),
        }
    }
}

impl_parse! {
    String = (StringAttribute as crate::kw::string),
    VarI32 = (VarI32Attribute as crate::kw::var32),
    VarI64 = (VarI64Attribute as crate::kw::var64),
}
