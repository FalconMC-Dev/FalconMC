use std::{collections::HashSet, hash::Hash};

use crate::{kw, util::FieldData};

use falcon_proc_util::ErrorCatcher;
use proc_macro2::Span;
use string::StringData;
use syn::{parse::Parse, punctuated::Punctuated, Field, Token};

mod string;

pub(crate) enum WriteAttributes {
    String(StringData),
    Unknown(Span),
}

impl Hash for WriteAttributes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for WriteAttributes {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Unknown(_), Self::Unknown(_)) => true,
            _ => false,
        }
    }
}

impl Eq for WriteAttributes {}

impl Parse for WriteAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::string) {
            input.parse::<kw::string>()?;
            Ok(Self::String(input.parse::<StringData>()?))
        } else {
            Ok(Self::Unknown(input.span()))
        }
    }
}

impl<'a> FieldData<'a> for WriteAttributes {
    fn parse<I>(current: &'a syn::Field, others: I) -> syn::Result<Option<Vec<Self>>>
    where
        I: Iterator<Item = &'a syn::Field>,
        Self: Sized,
    {
        let (result, error) = current
            .attrs
            .iter()
            .filter(|attribute| attribute.path.is_ident("falcon"))
            .map(|attribute| {
                attribute
                    .parse_args_with(Punctuated::<WriteAttributes, Token![,]>::parse_terminated)
            })
            .fold(
                (HashSet::new(), ErrorCatcher::new()),
                |(mut result, mut error), attributes| {
                    match attributes {
                        Ok(attributes) => {
                            for attribute in attributes {
                                match attribute {
                                    WriteAttributes::Unknown(_) => {}
                                    _ => {
                                        if result.contains(&attribute) {
                                            error.add_error(syn::Error::new(
                                                attribute.span(),
                                                "Attribute already defined",
                                            ))
                                        } else {
                                            result.insert(attribute);
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => error.add_error(err),
                    }
                    (result, error)
                },
            );

        error.emit()?;

        let result: Vec<Self> = result.into_iter().collect();

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    fn span(&self) -> proc_macro2::Span {
        match self {
            WriteAttributes::String(data) => data.span(),
            WriteAttributes::Unknown(span) => *span,
        }
    }

    fn to_tokenstream(self, field: &'a Field) -> Vec<syn::Stmt> {
        todo!()
    }
}
