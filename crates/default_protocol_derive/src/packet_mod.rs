use std::collections::HashMap;
use syn::Ident;
use crate::PacketStruct;

pub(crate) fn packet_structs_to_version_list(packet_structs: Vec<PacketStruct>) -> HashMap<i32, Vec<(Ident, i32)>> {
    let mut map = HashMap::new();
    for packet_struct in packet_structs {
        let struct_name = packet_struct.struct_name;
        for (packet_id, version_list) in packet_struct.versions {
            for version in version_list {
                let id_list = map.entry(version).or_insert_with(Vec::new);
                id_list.push((struct_name.clone(), packet_id));
            }
        }
    }
    map
}