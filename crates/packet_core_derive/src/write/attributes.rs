use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::{kw, util::FieldData};

use falcon_proc_util::ErrorCatcher;
use proc_macro2::Span;
use string::StringData;
use syn::{parse::Parse, punctuated::Punctuated, Field, Token};

mod string;

#[derive(Debug)]
pub(crate) enum WriteAttributes {
    String(StringData),
    Unknown(Span),
}

impl WriteAttributes {
    fn check<'a, I>(&self, fields: I) -> syn::Result<()>
    where
        I: Iterator<Item = &'a Field>,
    {
        match &self {
            WriteAttributes::String(_) => Ok(()),
            WriteAttributes::Unknown(_) => Ok(()),
        }
    }

    pub fn is_end(&self) -> bool {
        match self {
            WriteAttributes::String(_) => true,
            WriteAttributes::Unknown(_) => false,
        }
    }
}

impl<'a> FieldData<'a> for WriteAttributes {
    fn parse(
        current: &'a syn::Field,
        others: &Punctuated<syn::Field, syn::token::Comma>,
    ) -> syn::Result<Option<Vec<Self>>>
    where
        Self: Sized,
    {
        let (result, mut error) = current
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

        for attribute in &result {
            error.extend_error(attribute.check(others.iter().filter(|e| e.ident != current.ident)));
        }

        error.emit()?;

        let mut result: Vec<Self> = result.into_iter().collect();
        result.sort_unstable();

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

    fn to_tokenstream(self, field: syn::Expr) -> syn::Expr {
        match self {
            WriteAttributes::String(data) => data.to_tokenstream(field),
            WriteAttributes::Unknown(_) => field,
        }
    }
}

impl PartialOrd for WriteAttributes {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (WriteAttributes::String(_), WriteAttributes::Unknown(_)) => Some(Ordering::Greater),
            (WriteAttributes::Unknown(_), WriteAttributes::String(_)) => Some(Ordering::Less),
            _ => Some(Ordering::Equal),
        }
    }
}

impl Ord for WriteAttributes {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
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
