//! Useful macros to avoid writing an unnecessary amount of code.
//!
//! More information can be found in the [Protocol Specification](https://wiki.vg/Protocol#Data_types).
//!
//! # Example
//! The following packet struct has 2 integer fields (one being var_i32, the other being a normal i64)
//! and one String field (with a maximum length of 200), the last field is ignored.
//! ```rust
//! #[derive(PacketEncode, PacketDecode)]
//! pub struct PacketExampleStruct {
//!     #[var_int]
//!     dummy: i32,
//!     number: i64,
//!     #[max_length(200)]
//!     cool_thingy: String,
//!     #[skip]
//!     ignored_field: u16,
//! }

extern crate proc_macro;

use proc_macro::TokenStream as TokenStream1;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::Type::Path;
use syn::{parse_macro_input, Error, Field, Fields, ItemStruct, LitInt, Type, TypePath};

#[proc_macro_derive(PacketEncode, attributes(var_int, max_length, skip))]
pub fn derive_packet_encode(item: TokenStream1) -> TokenStream1 {
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    let impl_encoder = implement_encoder(&mut item_struct);

    quote!(
        #impl_encoder
    )
    .into()
}

#[proc_macro_derive(PacketDecode, attributes(var_int, max_length, skip))]
pub fn derive_packet_decode(item: TokenStream1) -> TokenStream1 {
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    let impl_decoder = implement_decoder(&mut item_struct);

    quote!(
        #impl_decoder
    )
    .into()
}

fn implement_encoder(item_struct: &mut ItemStruct) -> TokenStream {
    let struct_ident = &item_struct.ident;

    let encoded_fields = match parse_fields(&item_struct.fields, encode_field) {
        Ok(tokens) => tokens,
        Err(error) => return error,
    };

    quote!(
        impl ::falcon_core::network::packet::PacketEncode for #struct_ident {
            fn to_buf(&self, buf: &mut dyn ::falcon_core::network::buffer::PacketBufferWrite) {
                #encoded_fields
            }
        }
    )
}

fn implement_decoder(item_struct: &mut ItemStruct) -> TokenStream {
    let struct_ident = &item_struct.ident;

    let decoded_fields = match parse_fields(&item_struct.fields, decode_field) {
        Ok(tokens) => tokens,
        Err(error) => return error,
    };

    quote!(
        impl ::falcon_core::network::packet::PacketDecode for #struct_ident {
            fn from_buf(buf: &mut dyn ::falcon_core::network::buffer::PacketBufferRead) -> ::falcon_core::error::Result<Self> {
                Ok(#struct_ident {
                    #decoded_fields
                })
            }
        }
    )
}

fn encode_field(field: &Field) -> Result<TokenStream, TokenStream> {
    let name = &field.ident;
    let ty = &field.ty;
    let ty_path = match ty {
        Path(ty_path) => ty_path,
        _ => return Err(Error::new(ty.span(), "Unexpected field type!").to_compile_error()),
    };

    if search_for_skip(field)? {
        return Ok(quote!());
    }
    let mut special = None;
    if let Some(tokens) = search_for_nbt(field, name, ty, true)? {
        special = Some(tokens);
    }
    if let Some(tokens) = search_for_var_int(field, name, ty, ty_path, true)? {
        if special.is_none() {
            special = Some(tokens);
        }
    }
    if let Some(tokens) = search_for_string(field, name, ty, ty_path, true)? {
        if special.is_none() {
            special = Some(tokens);
        }
    }
    if let Some(tokens) = special {
        return Ok(tokens);
    }

    Ok(quote!(self.#name.to_buf(buf);))
}

fn decode_field(field: &Field) -> Result<TokenStream, TokenStream> {
    let name = &field.ident;
    let ty = &field.ty;
    let ty_path = match ty {
        Path(ty_path) => ty_path,
        _ => return Err(Error::new(ty.span(), "Unexpected field type!").to_compile_error()),
    };

    if search_for_skip(field)? {
        return Ok(quote!(#name: ::std::default::Default::default(),));
    }

    let mut special = None;
    if let Some(tokens) = search_for_nbt(field, name, ty, false)? {
        special = Some(tokens);
    }
    if let Some(tokens) = search_for_var_int(field, name, ty, ty_path, false)? {
        if special.is_none() {
            special = Some(tokens);
        }
    }
    if let Some(tokens) = search_for_string(field, name, ty, ty_path, false)? {
        if special.is_none() {
            special = Some(tokens);
        }
    }
    if let Some(tokens) = special {
        return Ok(tokens);
    }

    Ok(quote!(#name: <#ty as ::falcon_core::network::packet::PacketDecode>::from_buf(buf)?.into(),))
}

fn search_for_skip(field: &Field) -> Result<bool, TokenStream> {
    match field.attrs.iter().find(|a| a.path.is_ident("skip")) {
        Some(attr) => match attr.parse_meta() {
            Ok(meta) => match meta {
                syn::Meta::Path(_) => Ok(true),
                _ => Err(
                    Error::new(meta.span(), "Unexpected argument(s) for `skip` attribute!")
                        .to_compile_error(),
                ),
            },
            Err(err) => Err(err.to_compile_error()),
        },
        None => Ok(false),
    }
}

fn search_for_nbt<T: ToTokens + Sized>(
    field: &Field,
    name: &Option<T>,
    ty: &Type,
    is_encode: bool,
) -> Result<Option<TokenStream>, TokenStream> {
    match field.attrs.iter().find(|a| a.path.is_ident("nbt")) {
        Some(attr) => match attr.parse_meta() {
            Ok(meta) => match meta {
                syn::Meta::Path(_) => {}
                _ => {
                    return Err(Error::new(
                        meta.span(),
                        "Unexpected argument(s) for `nbt` attribute!",
                    )
                    .to_compile_error())
                }
            },
            Err(err) => return Err(err.to_compile_error()),
        },
        None => return Ok(None),
    };

    // TODO: add to documentation that nbt needs to be good nbt and serializable
    if is_encode {
        Ok(Some(quote!(
            buf.write_u8_array(&::fastnbt::ser::to_bytes(&self.#name).unwrap());
        )))
    } else {
        Err(Error::new(ty.span(), "We cannot serialize NBT data (yet)").to_compile_error())
    }
}

fn search_for_var_int<T: ToTokens + Sized>(
    field: &Field,
    name: &Option<T>,
    ty: &Type,
    ty_path: &TypePath,
    is_encode: bool,
) -> Result<Option<TokenStream>, TokenStream> {
    match field.attrs.iter().find(|a| a.path.is_ident("var_int")) {
        Some(attr) => match attr.parse_meta() {
            Ok(meta) => match meta {
                syn::Meta::Path(_) => {}
                _ => {
                    return Err(Error::new(
                        meta.span(),
                        "Unexpected argument(s) for `var_int` attribute!",
                    )
                    .to_compile_error())
                }
            },
            Err(err) => return Err(err.to_compile_error()),
        },
        None => return Ok(None),
    };

    if ty_path.path.is_ident("i32") {
        Ok(Some(if is_encode {
            quote!(buf.write_var_i32(self.#name);)
        } else {
            quote!(#name: buf.read_var_i32()?,)
        }))
    } else if ty_path.path.is_ident("i64") {
        Ok(Some(if is_encode {
            quote!(buf.write_var_i64(self.#name);)
        } else {
            quote!(#name: buf.read_var_i64()?,)
        }))
    } else {
        Err(Error::new(
            ty.span(),
            "`var_int` can only be applied to `i32` or `i64`!",
        )
        .to_compile_error())
    }
}

fn search_for_string<T: ToTokens + Sized>(
    field: &Field,
    name: &Option<T>,
    ty: &Type,
    ty_path: &TypePath,
    is_encode: bool,
) -> Result<Option<TokenStream>, TokenStream> {
    let max_length = match field.attrs.iter().find(|a| a.path.is_ident("max_length")) {
        Some(attr) => {
            if !ty_path.path.is_ident("String") {
                return Err(
                    Error::new(ty.span(), "`max_length` can only be applied to `String`")
                        .to_compile_error(),
                );
            }
            match attr.parse_args::<LitInt>() {
                Ok(data) => match data.base10_parse::<i32>() {
                    Ok(len) => len,
                    Err(err) => return Err(err.to_compile_error()),
                },
                Err(err) => return Err(err.to_compile_error()),
            }
        }
        None => {
            if !ty_path.path.is_ident("String") {
                return Ok(None);
            }
            32767
        }
    };

    if is_encode {
        Ok(Some(quote!(buf.write_string(&self.#name);)))
    } else {
        Ok(Some(quote!(#name: buf.read_string(#max_length)?,)))
    }
}

fn parse_fields<F: Fn(&Field) -> Result<TokenStream, TokenStream>>(
    fields: &Fields,
    func: F,
) -> Result<TokenStream, TokenStream> {
    let mut output = quote!();

    if let Fields::Named(ref fields) = fields {
        let operations = fields.named.iter().map(func);
        for operation in operations {
            match operation {
                Err(err) => return Err(err),
                Ok(stream) => output.append_all(stream),
            }
        }
    }

    Ok(output)
}
