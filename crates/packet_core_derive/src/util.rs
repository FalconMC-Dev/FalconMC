use falcon_proc_util::ErrorCatcher;
use indexmap::IndexSet;
use syn::punctuated::Punctuated;
use syn::{Field, Token};

use crate::attributes::PacketAttribute;

pub struct ParsedFields<'a> {
    pub fields: Vec<(&'a Field, Vec<PacketAttribute>)>,
}

impl<'a> ParsedFields<'a> {
    pub fn new<F>(fields: &'a Punctuated<Field, Token![,]>, validate: F) -> syn::Result<Self>
    where
        F: FnOnce(Vec<PacketAttribute>) -> syn::Result<Vec<PacketAttribute>> + Copy,
    {
        let mut result = Vec::with_capacity(fields.len());
        for field in fields {
            result.push((field, to_attributes(field, validate)?));
        }
        Ok(Self { fields: result })
    }
}

fn to_attributes<F>(field: &Field, validate: F) -> syn::Result<Vec<PacketAttribute>>
where
    F: FnOnce(Vec<PacketAttribute>) -> syn::Result<Vec<PacketAttribute>>,
{
    let mut error = ErrorCatcher::new();

    let attributes: Vec<PacketAttribute> = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("falcon"))
        .map(|a| a.parse_args_with(Punctuated::<PacketAttribute, Token![,]>::parse_terminated))
        .fold(IndexSet::new(), |mut result, attrs| {
            let attrs = attrs.map(|attrs| {
                for attr in attrs {
                    if result.contains(&attr) {
                        error.add_error(syn::Error::new(attr.span(), "Attribute already defined earlier"));
                    } else {
                        result.insert(attr);
                    }
                }
            });
            error.extend_error(attrs);
            result
        })
        .into_iter()
        .collect();

    error.emit()?;

    validate(attributes)
}
