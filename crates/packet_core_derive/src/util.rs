use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{Field, Token};

pub(crate) trait FieldData<'a> {
    fn parse(
        current: &'a Field,
        others: &Punctuated<Field, Token![,]>,
    ) -> syn::Result<Option<Vec<Self>>>
    where
        Self: Sized;

    fn span(&self) -> Span;

    fn to_tokenstream(self, field: syn::Expr) -> syn::Expr;
}

pub(crate) struct ParsedFields<'a, T>
where
    T: FieldData<'a>,
{
    pub fields: Vec<(&'a Field, Option<Vec<T>>)>,
}

impl<'a, T: FieldData<'a>> ParsedFields<'a, T> {
    pub(crate) fn new(fields: &'a Punctuated<Field, Token![,]>) -> syn::Result<Self> {
        let mut result = Vec::with_capacity(fields.len());
        for field in fields.iter() {
            result.push((field, T::parse(field, fields)?));
        }
        Ok(Self { fields: result })
    }
}
