use falcon_proc_util::ErrorCatcher;
use syn::{LitInt, Ident, Error};

#[derive(Debug)]
pub(crate) struct ReceiveMatchMappings {
    pub(crate) mappings: Vec<(ReceivePacketID, ReceiveMappings)>,
}

impl ReceiveMatchMappings {
    pub fn new() -> Self {
        Self {
            mappings: vec![],
        }
    }

    pub fn add_packet(&mut self, packet_ident: Ident, (mut exclude, mappings): (Option<(LitInt, Ident)>, Vec<(LitInt, Vec<(LitInt, bool)>)>)) -> syn::Result<()> {
        let mut error = ErrorCatcher::new();

        for (packet_id, mut new_mapping) in mappings {
            let mut entry = None;
            for (i, (id, mappings)) in self.mappings.iter_mut().enumerate() {
                if id.packet_id.base10_digits() == packet_id.base10_digits() {
                    entry = Some(i);
                    for (version, has_errored) in new_mapping.iter_mut() {
                        if let Some((v, e)) = mappings.versions.iter_mut().find_map(|(_, m)| {
                            m.iter_mut().find(|(v, _)| v.base10_digits() == version.base10_digits())
                        }) {
                            if !*e {
                                *e = true;
                                error.add_error(Error::new(v.span(), "same \"protocol_id\" - \"packet_id\" pair for different packet type"));
                            }
                            *has_errored = true;
                            error.add_error(Error::new(version.span(), "same \"protocol_id\" - \"packet_id\" pair for different packet type"));
                        }
                    }
                    break;
                }
            }

            if let Some(i) = entry {
                let (id, mappings) = &mut self.mappings[i];
                if id.exclude.is_some() {
                    error.add_error(Error::new(packet_id.span(), "Another packet type has marked this packet id as \"all version capture\""));
                    error.add_error(Error::new(id.packet_id.span(), "Another packet type has assigned versions to this packet id while this packet type has assigned all versions to this packet id (-1)"));
                } else {
                    mappings.versions.push((packet_ident.clone(), new_mapping));
                }
            } else {
                self.mappings.push((ReceivePacketID::new(packet_id, None), ReceiveMappings::new(vec![(packet_ident.clone(), new_mapping)])));
            }
        }

        if let Some((ref exclude_new, _)) = exclude {
            match self.mappings.iter().find(|(id, _)| id.packet_id.base10_digits() == exclude_new.base10_digits()) {
                Some((ref exclude_old, _)) => {
                    if exclude_old.exclude.is_some() {
                        error.add_error(Error::new(exclude_old.packet_id.span(), "Cannot have more than one \"exclude\" version"));
                        error.add_error(Error::new(exclude_new.span(), "Cannot have more than one \"exclude\" version"));
                    } else {
                        error.add_error(Error::new(exclude_new.span(), "This packet id is used by more than one packet type"));
                    }
                }
                None => {
                    let (id, exclude) = exclude.take().unwrap();
                    self.mappings.push((ReceivePacketID::new(id, Some(exclude)), ReceiveMappings::new(vec![])));
                }
            }
        }

        error.emit()?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ReceivePacketID {
    pub(crate) packet_id: LitInt,
    pub(crate) exclude: Option<Ident>,
}

impl ReceivePacketID {
    pub fn new(packet_id: LitInt, exclude: Option<Ident>) -> Self {
        Self {
            packet_id,
            exclude,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReceiveMappings {
    pub(crate) versions: Vec<(Ident, Vec<(LitInt, bool)>)>,
}

impl ReceiveMappings {
    pub fn new(versions: Vec<(Ident, Vec<(LitInt, bool)>)>) -> Self {
        Self {
            versions,
        }
    }
}
