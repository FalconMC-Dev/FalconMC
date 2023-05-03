use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{LitInt, Type};

use crate::data::PacketMappings;

/// # Invariants
///
/// All [`LitIntBool`] must always be within i32 bounds.
#[derive(Debug, Clone, Default)]
pub struct VersionMappings {
    pub(crate) packet_to_versions: HashMap<Type, PacketVersions>,
}

impl VersionMappings {
    pub fn validate_mappings(mappings: Vec<PacketMappings>) -> Self {
        let mut result: VersionMappings = Default::default();
        for packet_mapping in mappings {
            for mapping in packet_mapping.mappings {
                let packet_id = mapping.id.into();
                let packet_versions = result.packet_to_versions.entry(packet_mapping.ty.clone()).or_default();
                let mut unique = packet_versions.insert(packet_id, mapping.versions.into_iter().map(|e| e.into()));
                // check that different packets have different versions for the same id
                for version in &mut unique {
                    for (_, packet_versions) in
                        result.packet_to_versions.iter().filter(|&(t, _)| t != &packet_mapping.ty)
                    {
                        if packet_versions.has_id_version(&packet_id, version) && !version.is_true() {
                            emit_error!(version.span(), "Different packets share the same id for this version");
                            version.toggle();
                        }
                    }
                }
            }
        }
        result
    }
}

/// # Invariants
///
/// All [`LitIntBool`] must always be within i32 bounds.
#[derive(Debug, Clone, Default)]
pub(crate) struct PacketVersions {
    mappings: HashMap<LitIntBool, Vec<LitIntBool>>,
}

impl PacketVersions {
    pub fn insert<I>(&mut self, id: LitIntBool, versions: I) -> Vec<LitIntBool>
    where
        I: IntoIterator<Item = LitIntBool>,
    {
        let mut result = Vec::new();
        // Check that a packet has no two ids for the same version
        for mut version in versions {
            self.mappings
                .iter_mut()
                .filter(|(&p, _)| p != id)
                .for_each(|(_, versions)| {
                    if let Some(v) = versions.iter_mut().find(|v| v == &&version) {
                        if !v.is_true() {
                            emit_error!(v.span(), "The same packet has multiple ids for the this version");
                            v.toggle();
                        }
                        emit_error!(version.span(), "The same packet has multiple ids for the this version");
                        version.toggle();
                    }
                });
            if !version.is_true() {
                let mapping = self.mappings.entry(id).or_default();
                if !mapping.contains(&version) {
                    mapping.push(version);
                    result.push(version);
                }
            }
        }
        result
    }

    pub fn has_id_version(&self, id: &LitIntBool, version: &LitIntBool) -> bool {
        for (_, versions) in self.mappings.iter().filter(|&(p, _)| p == id) {
            if versions.iter().any(|v| v == version) {
                return true;
            }
        }
        false
    }

    pub fn into_mappings(self) -> HashMap<LitIntBool, Vec<LitIntBool>> { self.mappings }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LitIntBool {
    value: i32,
    span: Span,
    toggle: bool,
}

impl LitIntBool {
    pub fn is_true(&self) -> bool { self.toggle }

    pub fn toggle(&mut self) { self.toggle = !self.toggle }

    pub fn span(&self) -> Span { self.span }
}

impl ToTokens for LitIntBool {
    fn to_tokens(&self, tokens: &mut TokenStream) { LitInt::new(&self.value.to_string(), self.span).to_tokens(tokens) }
}

impl Deref for LitIntBool {
    type Target = i32;

    fn deref(&self) -> &Self::Target { &self.value }
}

impl From<LitInt> for LitIntBool {
    fn from(value: LitInt) -> Self {
        Self {
            value: value.base10_parse::<i32>().unwrap(),
            span: value.span(),
            toggle: false,
        }
    }
}

impl PartialEq for LitIntBool {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}

impl Eq for LitIntBool {}

impl PartialOrd for LitIntBool {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { self.value.partial_cmp(&other.value) }
}

impl Ord for LitIntBool {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.value.cmp(&other.value) }
}

impl Hash for LitIntBool {
    fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state); }
}
