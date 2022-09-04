use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{Field, Stmt, Token};

pub(crate) trait FieldData<'a> {
    fn parse<I>(current: &'a Field, others: I) -> syn::Result<Option<Vec<Self>>>
    where
        I: Iterator<Item = &'a Field>,
        Self: Sized;

    fn span(&self) -> Span;

    fn to_tokenstream(self, field: &'a Field) -> Vec<Stmt>;
}

pub(crate) struct ParsedFields<'a, T>
where
    T: FieldData<'a>,
{
    fields: Vec<(&'a Field, Option<Vec<T>>)>,
}

impl<'a, T: FieldData<'a>> ParsedFields<'a, T> {
    pub(crate) fn new(fields: &'a Punctuated<Field, Token![,]>) -> syn::Result<Self> {
        let mut result = Vec::with_capacity(fields.len());
        for field in fields.iter() {
            result.push((
                field,
                T::parse(field, fields.iter().filter(|e| e.ident != field.ident))?,
            ));
        }
        Ok(Self { fields: result })
    }
}
