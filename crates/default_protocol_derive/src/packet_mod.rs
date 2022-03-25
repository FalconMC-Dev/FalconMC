use std::collections::HashMap;
use syn::{Ident, LitStr};
use crate::PacketStruct;

pub(crate) fn packet_structs_to_version_receive_list(packet_structs: &[PacketStruct]) -> HashMap<i32, Vec<(Ident, i32)>> {
    let mut map = HashMap::new();
    for packet_struct in packet_structs {
        if !packet_struct.incoming {
            continue;
        }
        let struct_name = packet_struct.struct_name.clone();
        for (packet_id, version_list) in &packet_struct.versions {
            for version in version_list {
                let id_list = map.entry(*version).or_insert_with(Vec::new);
                id_list.push((struct_name.clone(), *packet_id));
            }
        }
    }
    map
}

pub(crate) fn packet_structs_to_version_outgoing_list(packet_structs: &[PacketStruct]) -> HashMap<Ident, (LitStr, (i32, Vec<i32>))> {
    let mut map = HashMap::new();
    for packet_struct in packet_structs {
        if let Some(name) = &packet_struct.outgoing {
            let struct_name = packet_struct.struct_name.clone();
            for (packet_id, version_list) in &packet_struct.versions {
                for version in version_list {
                    let (_, (_, id_list)) = map.entry(struct_name.clone()).or_insert_with(|| (name.clone(), (*packet_id, Vec::new())));
                    id_list.push(*version);
                }
            }
        }
    }
    map
}