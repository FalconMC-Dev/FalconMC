use falcon_proc_util::ErrorCatcher;
use falcon_protocol_util::{SendFnName, VersionsToID, PacketVersionMappings};
use proc_macro2::Ident;
use syn::{braced, Error, ItemStruct, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use crate::kw;

#[derive(Debug)]
pub(crate) struct PacketData {
    struct_name: Ident,
    fn_name: LitStr,
    batch_name: Option<LitStr>,
    versions: PacketVersionMappings,
}

impl PacketData {
    pub(crate) fn parse_packet(item: &mut ItemStruct) -> syn::Result<Option<PacketData>> {
        let mut error = ErrorCatcher::new();
        let mut fn_name = SendFnName::new();
        let mut batch_name = SendFnName::new();
        let mut versions = PacketVersionMappings::new();
        let mut found = false;

        for attr in &item.attrs {
            if attr.path.is_ident("falcon_packet") {
                found = true;
                let args = attr.parse_args_with(Punctuated::<PacketAttributes, Token![,]>::parse_terminated)?;
                for arg in args {
                    match arg {
                        PacketAttributes::Name(n) => error.extend_error(fn_name.set_name(n)),
                        PacketAttributes::Versions(v) => error.extend_error(versions.add_versions(v.into_iter())),
                        PacketAttributes::Batched(b) => error.extend_error(batch_name.set_name(b)),
                    }
                }
            }
        }

        item.attrs.retain(|attr| !attr.path.is_ident("falcon_packet"));
        error.emit()?;
        
        if found {
            Ok(Some(PacketData {
                struct_name: item.ident.clone(),
                fn_name: fn_name.name().ok_or_else(|| Error::new(item.ident.span(), "missing \"falcon_packet\" attribute \"name\""))?,
                batch_name: batch_name.name(),
                versions,
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn mappings(&self) -> &PacketVersionMappings {
        &self.versions
    }

    pub(crate) fn fn_name(&self) -> &LitStr {
        &self.fn_name
    }

    pub(crate) fn struct_name(&self) -> &Ident {
        &self.struct_name
    }

    pub(crate) fn batch_name(&self) -> Option<&LitStr> {
        self.batch_name.as_ref()
    }
}

#[derive(Debug)]
enum PacketAttributes {
    Versions(Punctuated<VersionsToID, Token![;]>),
    Name(LitStr),
    Batched(LitStr),
}

impl Parse for PacketAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw::versions) {
            input.parse::<kw::versions>()?;
            input.parse::<Token![=]>()?;
            let content;
            braced!(content in input);
            Ok(Self::Versions(content.parse_terminated(VersionsToID::parse)?))
        } else if input.peek(kw::name) {
            input.parse::<kw::name>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::Name(input.parse()?))
        } else if input.peek(kw::batching) {
            input.parse::<kw::batching>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::Batched(input.parse()?))
        } else {
            Err(Error::new(input.span(), "Unexpected attribute argument!"))
        }
    }
}

