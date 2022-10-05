use std::str::FromStr;

use falcon_proc_util::ErrorCatcher;
use itertools::Itertools;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, LitInt, Token};

#[derive(Debug)]
pub struct PacketVersionMappings {
    versions: Vec<(LitInt, Vec<(LitInt, bool)>)>,
    /// TODO: change into Option<LitInt>
    is_exclude: Option<LitInt>,
}

impl PacketVersionMappings {
    pub fn new() -> Self {
        Self {
            versions: vec![],
            is_exclude: None,
        }
    }

    pub fn add_versions<I>(&mut self, versions: I) -> syn::Result<()>
    where
        I: Iterator<Item = VersionsToID>,
    {
        let mut error = ErrorCatcher::new();

        for new_version in versions {
            let new_version = new_version.check_duplicates()?;

            if let Some(v) = new_version.find_version(-1) {
                if self.is_exclude.is_some() {
                    error.add_error(Error::new(v.span(), "-1 already used elsewhere"));
                }
                self.is_exclude = Some(new_version.id);
                continue;
            }

            let mut entry = None;
            for (i, (packet_id, versions)) in self.versions.iter_mut().enumerate() {
                if packet_id.base10_digits() == new_version.id.base10_digits() {
                    entry = Some(i);
                }
                for version in &new_version.versions {
                    if let Some((old_version, has_errored)) = versions.iter_mut().find(|(v, _)| v.base10_digits() == version.base10_digits()) {
                        if !*has_errored {
                            *has_errored = true;
                            error.add_error(Error::new(old_version.span(), "duplicate protocol version"));
                        }
                        error.add_error(Error::new(version.span(), "duplicate protocol version"));
                    }
                }
            }
            if let Some(i) = entry {
                self.versions[i].1.extend(new_version.versions.into_iter().map(|v| (v, false)));
            } else {
                match &self.is_exclude {
                    Some(exclude) => {
                        if exclude.base10_digits() != new_version.id.base10_digits() {
                            self.versions
                                .push((new_version.id, new_version.versions.into_iter().map(|v| (v, false)).collect()));
                        }
                    },
                    _ => self
                        .versions
                        .push((new_version.id, new_version.versions.into_iter().map(|v| (v, false)).collect())),
                }
            }
        }

        error.emit()?;
        Ok(())
    }

    pub fn versions(&self) -> impl Iterator<Item = (&LitInt, Vec<&LitInt>)> {
        self.versions
            .iter()
            .map(|(id, versions)| (id, versions.iter().map(|(v, _)| v).collect()))
    }

    pub fn is_exclude(&self) -> Option<&LitInt> { self.is_exclude.as_ref() }

    pub fn to_inner(&self) -> (Option<LitInt>, Vec<(LitInt, Vec<(LitInt, bool)>)>) { (self.is_exclude.clone(), self.versions.clone()) }
}

#[derive(Debug)]
pub struct VersionsToID {
    versions: Punctuated<LitInt, Token![,]>,
    id: LitInt,
}

impl VersionsToID {
    pub(crate) fn check_duplicates(self) -> syn::Result<Self> {
        let versions: Punctuated<LitInt, Token![,]> = self
            .versions
            .into_iter()
            .sorted_by(|v1, v2| v1.base10_digits().cmp(v2.base10_digits()))
            .collect();

        let mut error = ErrorCatcher::new();

        let mut started = false;
        {
            let mut iterator = versions.iter().peekable();
            while let Some(element) = iterator.next() {
                if let Some(other) = iterator.peek() {
                    if element.base10_digits() == other.base10_digits() {
                        if !started {
                            started = true;
                            error.add_error(Error::new(element.span(), "duplicate protocol version"))
                        }
                        error.add_error(Error::new(other.span(), "duplicate protocol version"))
                    } else {
                        started = false;
                    }
                }
            }
        }

        error.emit()?;
        Ok(VersionsToID {
            versions,
            id: self.id,
        })
    }

    pub fn find_version<N>(&self, version: N) -> Option<&LitInt>
    where
        N: FromStr,
        N: PartialEq<N>,
        <N as FromStr>::Err: std::fmt::Display,
    {
        self.versions
            .iter()
            .find(|v| v.base10_parse::<N>().ok().map(|n| n == version).unwrap_or(false))
    }
}

impl Parse for VersionsToID {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let versions = Punctuated::<LitInt, Token![,]>::parse_separated_nonempty(input)?;
        input.parse::<Token![=]>()?;
        let id = input.parse::<LitInt>()?;
        Ok(VersionsToID { versions, id })
    }
}
