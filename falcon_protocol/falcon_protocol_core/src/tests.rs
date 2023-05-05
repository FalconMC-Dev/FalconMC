#![cfg(test)]

use pretty_assertions::assert_eq;
use syn::parse::Parser;
use syn::parse_quote;

use crate::check::VersionMappings;
use crate::data::parse_mappings;
use crate::generate::generate_output;

#[test]
fn test_packets() {
    let input = parse_quote! {
        PacketOne: 47, 100, 120 = 0x01,
        PacketTwo: { 49, 20 = 0x01; 100 = 0x02 },
    };
    let syntax = parse_mappings.parse2(input).unwrap();
    let mappings = VersionMappings::validate_mappings(syntax);
    let tokens = generate_output(mappings);

    assert_eq!(
        "impl :: falcon_protocol :: Packet for PacketOne { } impl :: falcon_protocol :: Packet for PacketTwo { } pub \
         fn read_packet < B > (buffer : & mut B , packet_id : i32 , protocol_version : i32) -> :: std :: result :: \
         Result < std :: option :: Option < Box < dyn :: falcon_protocol :: Packet >> , :: falcon_packet :: ReadError \
         > where B : :: bytes :: Buf , { Ok (match packet_id { 1 => { match protocol_version { 47 | 100 | 120 => Some \
         (:: std :: boxed :: Box :: new (< PacketOne as :: falcon_packet :: PacketRead > :: read (buffer) ?)) , 49 | \
         20 => Some (:: std :: boxed :: Box :: new (< PacketTwo as :: falcon_packet :: PacketRead > :: read (buffer) \
         ?)) , _ => None , } } 2 => { match protocol_version { 100 => Some (:: std :: boxed :: Box :: new (< \
         PacketTwo as :: falcon_packet :: PacketRead > :: read (buffer) ?)) , _ => None , } } _ => None , }) }",
        tokens.to_string()
    );
}
