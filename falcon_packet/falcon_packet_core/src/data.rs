use proc_macro2::TokenStream;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::{Brace, Colon, Comma, Eq, Let, Paren, Struct};
use syn::{braced, parenthesized, Attribute, Block, Expr, FnArg, Ident, LitInt, Stmt, Token, Type, Visibility};

use crate::kw;

#[derive(Debug)]
pub struct ValiditySyntax {
    pub has_packet: bool,
}

impl Parse for ValiditySyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.call(Attribute::parse_outer)?;
        input.parse::<Visibility>()?;
        let has_packet = if input.peek(kw::packet) {
            input.parse::<kw::packet>()?;
            true
        } else {
            false
        };
        input.parse::<Token![struct]>()?;
        input.parse::<TokenStream>()?;
        Ok(Self { has_packet })
    }
}

pub struct StructSyntax {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Struct,
    pub packet_name: Ident,
    pub inputs: Punctuated<FnArg, Token![,]>,
    pub brace_token: Brace,
    pub init: Vec<Stmt>,
    pub fields: Punctuated<PacketStructField, Comma>,
}

impl Parse for StructSyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;
        let struct_token = input.parse::<Token![struct]>()?;
        let packet_name = input.parse::<Ident>()?;
        let inputs = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            input.call(Punctuated::<FnArg, Token![,]>::parse_separated_nonempty)?
        } else {
            Punctuated::new()
        };
        let content;
        let brace_token = braced!(content in input);
        let init = if content.peek(kw::init) {
            content.parse::<kw::init>()?;
            content.parse::<Token![=]>()?;
            let init_content;
            braced!(init_content in content);
            init_content.call(Block::parse_within)?
        } else {
            Vec::new()
        };
        let fields = Punctuated::<PacketStructField, Comma>::parse_terminated(&content)?;
        Ok(Self {
            attrs,
            vis,
            struct_token,
            packet_name,
            inputs,
            brace_token,
            init,
            fields,
        })
    }
}

pub struct PacketSyntax {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub packet_token: kw::packet,
    pub struct_token: Struct,
    pub packet_name: Ident,
    pub inputs: Punctuated<FnArg, Token![,]>,
    pub brace_token: Brace,
    pub init: Vec<Stmt>,
    pub fields: Punctuated<PacketField, Comma>,
}

impl Parse for PacketSyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;
        let packet_token = input.parse::<kw::packet>()?;
        let struct_token = input.parse::<Token![struct]>()?;
        let packet_name = input.parse::<Ident>()?;
        let inputs = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            input.call(Punctuated::<FnArg, Token![,]>::parse_separated_nonempty)?
        } else {
            Punctuated::new()
        };
        let content;
        let brace_token = braced!(content in input);
        let init = if content.peek(kw::init) {
            content.parse::<kw::init>()?;
            content.parse::<Token![=]>()?;
            let init_content;
            braced!(init_content in content);
            init_content.call(Block::parse_within)?
        } else {
            Vec::new()
        };
        let fields = Punctuated::<PacketField, Comma>::parse_terminated(&content)?;
        Ok(Self {
            attrs,
            vis,
            packet_token,
            struct_token,
            packet_name,
            inputs,
            brace_token,
            init,
            fields,
        })
    }
}

pub struct PacketField {
    pub spec: FieldSpec,
    pub struct_field: PacketStructField,
}

impl Parse for PacketField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (spec, field) = parse_field(input)?;
        Ok(PacketField {
            spec,
            struct_field: field,
        })
    }
}

pub fn parse_field<T>(input: syn::parse::ParseStream) -> syn::Result<(FieldSpec, T)>
where
    T: Parse,
{
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![self]) {
        input.parse::<Token![self]>()?;
        if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            let spec = FieldSpec::DirectAs(input.parse::<Type>()?);
            input.parse::<Token![=>]>()?;
            Ok((spec, input.parse()?))
        } else {
            input.parse::<Token![=>]>()?;
            Ok((FieldSpec::Direct, input.parse()?))
        }
    } else if lookahead.peek(kw::var32) {
        input.parse::<kw::var32>()?;
        input.parse::<Token![=>]>()?;
        Ok((FieldSpec::Var32, input.parse()?))
    } else if lookahead.peek(kw::var64) {
        input.parse::<kw::var64>()?;
        input.parse::<Token![=>]>()?;
        Ok((FieldSpec::Var64, input.parse()?))
    } else if lookahead.peek(kw::str) {
        input.parse::<kw::str>()?;
        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let spec = FieldSpec::String(content.parse::<LitInt>()?.base10_parse::<usize>()?);
            input.parse::<Token![=>]>()?;
            Ok((spec, input.parse()?))
        } else {
            input.parse::<Token![=>]>()?;
            Ok((FieldSpec::String(32767), input.parse()?))
        }
    } else if lookahead.peek(kw::bytes) {
        input.parse::<kw::bytes>()?;
        input.parse::<Token![=>]>()?;
        let content;
        braced!(content in input);
        let struct_field = content.parse()?;
        content.parse::<Token![,]>()?;
        let ident = content.parse()?;
        content.parse::<Token![=]>()?;
        Ok((FieldSpec::Bytes((ident, content.parse()?)), struct_field))
    } else if lookahead.peek(kw::rest) {
        input.parse::<kw::rest>()?;
        input.parse::<Token![=>]>()?;
        Ok((FieldSpec::Rest, input.parse()?))
    } else if lookahead.peek(kw::array) {
        input.parse::<kw::array>()?;
        input.parse::<Token![=>]>()?;
        Ok((FieldSpec::Array, input.parse()?))
    } else if lookahead.peek(kw::bytearray) {
        input.parse::<kw::bytearray>()?;
        input.parse::<Token![=>]>()?;
        Ok((FieldSpec::ByteArray, input.parse()?))
    } else if lookahead.peek(kw::nbt) {
        input.parse::<kw::nbt>()?;
        input.parse::<Token![=>]>()?;
        Ok((FieldSpec::Nbt, input.parse()?))
    } else {
        return Err(lookahead.error());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSpec {
    Direct,
    DirectAs(Type),
    Var32,
    Var64,
    String(usize),
    Bytes((Ident, Expr)),
    Rest,
    Array,
    ByteArray,
    Nbt,
}

pub struct PacketStructField {
    pub let_token: Option<Let>,
    pub ident: Ident,
    pub colon_token: Colon,
    pub ty: Type,
    pub eq_token: Option<Eq>,
    pub init: Option<Expr>,
}

impl Parse for PacketStructField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_init = input.peek(Let);
        let let_token = is_init.then(|| input.parse()).transpose()?;
        let ident = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;
        let eq_token = is_init.then(|| input.parse()).transpose()?;
        let init = is_init.then(|| input.parse()).transpose()?;
        Ok(Self {
            let_token,
            ident,
            colon_token,
            ty,
            eq_token,
            init,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_struct_field() {
        let struct_field: PacketStructField = parse_quote! {
            let x: i32 = { let temp = 1; temp * 3 }
        };
        assert!(struct_field.let_token.is_some());
        assert!(struct_field.eq_token.is_some());
        assert!(struct_field.init.is_some());

        let struct_field: PacketStructField = parse_quote! {
            y: i32
        };
        assert!(struct_field.let_token.is_none());
        assert!(struct_field.eq_token.is_none());
        assert!(struct_field.init.is_none());
    }

    #[test]
    fn test_packet_field() {
        let field_self: PacketField = parse_quote! {
            self => y: i32
        };
        assert_eq!(FieldSpec::Direct, field_self.spec);
        let field_self_as: PacketField = parse_quote! {
            self as i64 => y: i32
        };
        assert_eq!(FieldSpec::DirectAs(parse_quote!(i64)), field_self_as.spec);
        let field_var32: PacketField = parse_quote! {
            var32 => y: i32
        };
        assert_eq!(FieldSpec::Var32, field_var32.spec);
        let field_var64: PacketField = parse_quote! {
            var64 => y: i32
        };
        assert_eq!(FieldSpec::Var64, field_var64.spec);
        let field_str_default: PacketField = parse_quote! {
            str => z: PacketString
        };
        assert_eq!(FieldSpec::String(32767), field_str_default.spec);
        let field_str: PacketField = parse_quote! {
            str(32) => z: PacketString
        };
        assert_eq!(FieldSpec::String(32), field_str.spec);
        let field_bytes: PacketField = parse_quote! {
            bytes => { u: PacketBytes, u_len = u.len() }
        };
        assert_eq!(FieldSpec::Bytes((parse_quote!(u_len), parse_quote! {u.len()})), field_bytes.spec);
        let field_rest: PacketField = parse_quote! {
            rest => u: PacketBytes
        };
        assert_eq!(FieldSpec::Rest, field_rest.spec);
        let field_array: PacketField = parse_quote! {
            array => u: [i32; 3]
        };
        assert_eq!(FieldSpec::Array, field_array.spec);
        let field_bytearray: PacketField = parse_quote! {
            bytearray => u: [i32; 3]
        };
        assert_eq!(FieldSpec::ByteArray, field_bytearray.spec);
        let field_nbt: PacketField = parse_quote! {
            nbt => u: MyStruct
        };
        assert_eq!(FieldSpec::Nbt, field_nbt.spec);
    }

    #[test]
    fn test_packet_syntax() {
        let _packet: PacketSyntax = parse_quote! {
            #[derive(Debug)]
            #[test]
            pub packet struct PacketTest => server: &Server {
                init = {
                    let player = server.player(uuid);
                }
                str(16) => let name: PacketString = player.name(),
                self => uuid: Uuid,
            }
        };
    }
}
