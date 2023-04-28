use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::{braced, parse2, parse_quote, Expr, Ident, Token, Type};

use crate::processing::{field_to_read, field_to_size, field_to_write};
use crate::{parse_field, FieldSpec};

pub fn packet_write(input: TokenStream) -> TokenStream {
    let syntax = match parse2::<OutPacketSyntax>(input) {
        Ok(syntax) => syntax,
        Err(err) => return err.to_compile_error(),
    };

    let fields_preprocess = syntax.fields.iter().filter_map(|f| match f.spec {
        FieldSpec::Bytes((ref ident, ref expr)) => Some(quote!(let #ident = #expr;)),
        _ => None,
    });

    let buffer = syntax.buffer.unwrap_or_else(|| parse_quote!(buffer));
    let writes = syntax.fields.iter().map(|field| {
        let value = &field.field.value;
        field_to_write(&field.field.ty, &field.spec, quote!(#value), &buffer)
    });

    quote!(#(#fields_preprocess)* #(#writes)*)
}

pub fn packet_read(input: TokenStream) -> TokenStream {
    let syntax = match parse2::<LetPacketSyntax>(input) {
        Ok(syntax) => syntax,
        Err(err) => return err.to_compile_error(),
    };

    let buffer = syntax.buffer.unwrap_or_else(|| parse_quote!(buffer));
    let reads = syntax
        .fields
        .iter()
        .map(|field| field_to_read(&field.field.ty, &field.spec, &field.field.name, &buffer));

    quote!(#(#reads)*)
}

pub fn packet_size(input: TokenStream) -> TokenStream {
    let syntax = match parse2::<OutPacketSyntax>(input) {
        Ok(syntax) => syntax,
        Err(err) => return err.to_compile_error(),
    };

    let fields_preprocess = syntax.fields.iter().filter_map(|f| match f.spec {
        FieldSpec::Bytes((ref ident, ref expr)) => Some(quote!(let #ident = #expr;)),
        _ => None,
    });
    let sizes = syntax.fields.iter().map(|field| {
        let value = &field.field.value;
        field_to_size(&field.field.ty, &field.spec, quote!(#value))
    });

    quote!(#(#fields_preprocess)* #(#sizes)+*)
}

fn parse_trait<T: Parse>(input: ParseStream) -> syn::Result<(Option<Ident>, Punctuated<T, Token![,]>)> {
    let content;
    if input.peek(Ident) && input.peek2(Token![=>]) && input.peek3(Brace) {
        let buffer = input.parse::<Ident>()?;
        input.parse::<Token![=>]>()?;
        braced!(content in input);
        let fields = Punctuated::<T, Token![,]>::parse_terminated(&content)?;
        return Ok((Some(buffer), fields));
    }
    let fields = Punctuated::<T, Token![,]>::parse_terminated(input)?;
    Ok((None, fields))
}

pub struct OutPacketSyntax {
    pub buffer: Option<Ident>,
    pub fields: Punctuated<OutPacketField, Token![,]>,
}

impl Parse for OutPacketSyntax {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (buffer, fields) = parse_trait(input)?;
        Ok(OutPacketSyntax { buffer, fields })
    }
}

pub struct OutPacketField {
    pub spec: FieldSpec,
    pub field: OutField,
}

impl Parse for OutPacketField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (spec, field) = parse_field(input)?;
        Ok(OutPacketField { spec, field })
    }
}

pub struct OutField {
    pub ty: Type,
    pub value: Expr,
}

impl Parse for OutField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(OutField { ty, value })
    }
}

pub struct LetPacketSyntax {
    pub buffer: Option<Ident>,
    pub fields: Punctuated<LetPacketField, Token![,]>,
}

impl Parse for LetPacketSyntax {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (buffer, fields) = parse_trait(input)?;
        Ok(LetPacketSyntax { buffer, fields })
    }
}

pub struct LetPacketField {
    pub spec: FieldSpec,
    pub field: VarField,
}

impl Parse for LetPacketField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (spec, field) = parse_field(input)?;
        Ok(LetPacketField { spec, field })
    }
}

pub struct VarField {
    pub name: Ident,
    pub ty: Type,
}

impl Parse for VarField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { name, ty })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outfield() { let _field: OutPacketField = parse_quote!(var32 => i32 = num); }

    #[test]
    fn test_out_syntax() { let _syntax: OutPacketSyntax = parse_quote!(var32 => i32 = num); }

    #[test]
    fn test_size() {
        let s = packet_size(quote!(self => i32 = field1));
        println!("{}", s);
    }
}
