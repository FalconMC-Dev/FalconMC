use proc_macro_error::{abort, emit_error};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, Error, Ident, LitInt, Token, Type};

use crate::check::LitIntBool;

pub fn parse_mappings(input: ParseStream) -> syn::Result<Vec<PacketMappings>> {
    let mut mappings = Vec::new();
    while input.peek(Ident) {
        mappings.push(input.parse::<PacketMappings>()?);
    }
    if !input.is_empty() {
        abort! { input.span(), "Unexpected tokens" };
    }
    Ok(mappings)
}

pub struct PacketMappings {
    pub ty: Type,
    pub colon: Token![:],
    pub brace: Option<Brace>,
    pub mappings: Punctuated<VersionsToId, Token![;]>,
    pub comma: Option<Token![,]>,
}

impl Parse for PacketMappings {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Type>()?;
        let colon = input.parse::<Token![:]>()?;
        let brace;
        let mappings;
        if input.peek(Brace) {
            let content;
            brace = Some(braced!(content in input));
            mappings = Punctuated::<VersionsToId, Token![;]>::parse_terminated(&content)?;
        } else {
            brace = None;
            mappings = Punctuated::<VersionsToId, Token![;]>::parse_separated_nonempty(input)?;
        }
        if mappings.len() > 1 && brace.is_none() {
            emit_error! { mappings.pairs().next().unwrap().punct().unwrap(),
                "Multiple mappings require surrounding by braces"
            }
        }
        let comma = if (mappings.len() > 1 && input.peek(Token![,])) || input.peek(Ident) || input.peek(Token![,]) {
            Some(input.parse::<Token![,]>()?)
        } else {
            None
        };

        Ok(Self {
            ty: ident,
            colon,
            brace,
            mappings,
            comma,
        })
    }
}

/// # Invariants
///
/// A `LitInt` **must** always be within i32 bounds.
/// The id `LitInt` **must** always be within i32 bounds.
pub struct VersionsToId {
    pub versions: VersionsOrAll,
    pub eq: Token![=],
    pub id: LitInt,
}

impl Parse for VersionsToId {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let versions = if input.peek(Token![_]) {
            VersionsOrAll::All(input.parse::<Token![_]>()?)
        } else {
            let versions = Punctuated::<LitInt, Token![,]>::parse_separated_nonempty(input)?;
            // check that all versions are within i32 bounds
            // errors are aggregated
            let mut errors: Option<Error> = None;
            for version in &versions {
                if let Err(error) = version.base10_parse::<i32>() {
                    if let Some(errors) = &mut errors {
                        errors.combine(error);
                    } else {
                        errors = Some(error)
                    }
                }
            }
            if let Some(error) = errors {
                return Err(error);
            }
            VersionsOrAll::Versions(versions)
        };
        let eq = input.parse::<Token![=]>()?;
        let id = input.parse::<LitInt>()?;
        // Check that id is within i32 bounds
        id.base10_parse::<i32>()?;
        Ok(Self { versions, eq, id })
    }
}

pub enum VersionsOrAll {
    Versions(Punctuated<LitInt, Token![,]>),
    All(Token![_]),
}

impl VersionsOrAll {
    #[cfg(test)]
    pub fn len(&self) -> usize {
        match self {
            VersionsOrAll::Versions(v) => v.len(),
            VersionsOrAll::All(_) => 1,
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = LitIntBool> {
        match self {
            VersionsOrAll::Versions(v) => VersionsIter::Versions(v.into_iter().map(|e| e.into())),
            VersionsOrAll::All(s) => VersionsIter::All(std::iter::once(s.span().into())),
        }
    }
}

pub enum VersionsIter<I> {
    Versions(I),
    All(std::iter::Once<LitIntBool>),
}

impl<I> Iterator for VersionsIter<I>
where
    I: Iterator<Item = LitIntBool>,
{
    type Item = LitIntBool;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            VersionsIter::Versions(v) => v.next(),
            VersionsIter::All(v) => v.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;
    use crate::check::LitIntValue;

    #[test]
    fn test_versions_to_id() {
        let syntax: VersionsToId = parse_quote! {
            47, 23, 35 = 0x02
        };
        assert_eq!(0x02, syntax.id.base10_parse::<i32>().unwrap());
        let mut iter = syntax.versions.into_iter();
        assert_eq!(LitIntValue::Number(47), iter.next().unwrap().value);
        assert_eq!(LitIntValue::Number(23), iter.next().unwrap().value);
        assert_eq!(LitIntValue::Number(35), iter.next().unwrap().value);
    }

    #[test]
    fn test_single_mapping_trailing() {
        let syntax: PacketMappings = parse_quote! {
            ExamplePacket: 23, 35 = 0x03,
        };
        assert_eq!(2, syntax.mappings.first().unwrap().versions.len());
        assert_eq!(3, syntax.mappings.first().unwrap().id.base10_parse::<i32>().unwrap());
    }

    #[test]
    fn test_single_mapping_notrailing() {
        let syntax: PacketMappings = parse_quote! {
            ExamplePacket: 23, 35 = 0x03
        };
        assert_eq!(1, syntax.mappings.len());
    }

    #[test]
    fn test_multiple_trailing() {
        let syntax: PacketMappings = parse_quote! {
            PacketThree: {
                23, 22 = 0x05;
                88, 101 = 0x01;
            },
        };
        assert_eq!(2, syntax.mappings.len());
    }

    #[test]
    fn test_multiple_notrailing() {
        let syntax: PacketMappings = parse_quote! {
            PacketThree: {
                23, 22 = 0x05;
                88, 101 = 0x01
            }
        };
        assert_eq!(2, syntax.mappings.len());
    }

    #[test]
    #[should_panic]
    fn test_versions_too_large() {
        let _syntax: PacketMappings = parse_quote! {
            PacketThree: {
                23000000000000, 22, 3000000000000000 = 0x05;
            }
        };
    }

    #[test]
    #[should_panic]
    fn test_id_too_large() {
        let _syntax: PacketMappings = parse_quote! {
            PacketThree: {
                23, 22, 30 = 5660000000000000;
            }
        };
    }

    #[test]
    fn test_infer_syntax() {
        let _syntax: PacketMappings = parse_quote! {
            PacketOne: _ = 0x02,
        };
    }

    #[test]
    #[should_panic]
    fn test_infer_syntax_wrong() {
        let _syntax: PacketMappings = parse_quote! {
            PacketOne: _, 2 = 0x02,
        };
    }
}
