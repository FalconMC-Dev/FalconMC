use falcon_proc_util::ErrorCatcher;
use falcon_protocol_util::{PacketVersionMappings, VersionsToID};
use proc_macro2::Ident;
use syn::punctuated::Punctuated;
use syn::{ItemStruct, Token, braced};
use syn::parse::Parse;

use crate::kw;

pub(crate) struct PacketData {
    struct_name: Ident,
    versions: PacketVersionMappings,
}

impl PacketData {
    pub(crate) fn parse_packet(item: &mut ItemStruct) -> syn::Result<Option<PacketData>> {
        let mut error = ErrorCatcher::new();
        let mut versions = PacketVersionMappings::new();
        let mut found = false;

        for attr in &item.attrs {
            if attr.path.is_ident("falcon_packet") {
                found = true;
                let v = attr.parse_args::<VersionsArg>()?;
                error.extend_error(versions.add_versions(v.versions.into_iter()));
            }
        }

        item.attrs.retain(|attr| !attr.path.is_ident("falcon_packet"));
        error.emit()?;
        
        if found {
            Ok(Some(PacketData {
                struct_name: item.ident.clone(),
                versions,
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn mappings(&self) -> &PacketVersionMappings {
        &self.versions
    }

    pub(crate) fn struct_name(&self) -> &Ident {
        &self.struct_name
    }
}

pub(crate) struct VersionsArg {
    versions: Punctuated<VersionsToID, Token![;]>,
}

impl Parse for VersionsArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::versions>()?;
        input.parse::<Token![=]>()?;
        let content;
        braced!(content in input);
        Ok(Self {
            versions: content.parse_terminated(VersionsToID::parse)?
        })
    }
}
