use proc_macro_error::{abort, emit_error};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::{braced, Error, Ident, LitInt, Token, Type};

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
    pub versions: Punctuated<LitInt, Token![,]>,
    pub eq: Token![=],
    pub id: LitInt,
}

impl Parse for VersionsToId {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
        let eq = input.parse::<Token![=]>()?;
        let id = input.parse::<LitInt>()?;
        // Check that id is within i32 bounds
        id.base10_parse::<i32>()?;
        Ok(Self { versions, eq, id })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_versions_to_id() {
        let syntax: VersionsToId = parse_quote! {
            47, 23, 35 = 0x02
        };
        assert_eq!(0x02, syntax.id.base10_parse::<i32>().unwrap());
        let mut iter = syntax.versions.into_iter();
        assert_eq!(47, iter.next().unwrap().base10_parse::<i32>().unwrap());
        assert_eq!(23, iter.next().unwrap().base10_parse::<i32>().unwrap());
        assert_eq!(35, iter.next().unwrap().base10_parse::<i32>().unwrap());
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
}
