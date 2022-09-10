use falcon_proc_util::ErrorCatcher;
use indexmap::IndexSet;
use syn::punctuated::Punctuated;
use syn::{Error, Field, Token};

use crate::attributes::PacketAttribute;

pub struct ParsedFields<'a> {
    pub fields: Vec<(&'a Field, Vec<PacketAttribute>)>,
}

impl<'a> ParsedFields<'a> {
    pub fn new(fields: &'a Punctuated<Field, Token![,]>) -> syn::Result<Self> {
        let mut result = Vec::with_capacity(fields.len());
        for field in fields {
            result.push((field, to_attributes(field)?));
        }
        Ok(Self { fields: result })
    }
}

fn to_attributes(field: &Field) -> syn::Result<Vec<PacketAttribute>> {
    let mut error = ErrorCatcher::new();

    let mut attributes: Vec<PacketAttribute> = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("falcon"))
        .map(|a| a.parse_args_with(Punctuated::<PacketAttribute, Token![,]>::parse_terminated))
        .fold(IndexSet::new(), |mut result, attrs| {
            let attrs = attrs.map(|attrs| {
                for attr in attrs {
                    if result.contains(&attr) {
                        error.add_error(syn::Error::new(
                            attr.span(),
                            "Attribute already defined earlier",
                        ));
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

    let mut checked = Vec::with_capacity(attributes.len());

    for _ in 0..attributes.len() {
        let mut attribute = attributes.remove(0);
        error.extend_error(attribute.check(attributes.iter_mut()));
        if attribute.is_outer() && !attributes.is_empty() {
            error.add_error(Error::new(
                attribute.span(),
                "Ending attribute should be last in the list",
            ));
        }
        checked.push(attribute);
    }

    error.emit()?;

    Ok(checked)
}
