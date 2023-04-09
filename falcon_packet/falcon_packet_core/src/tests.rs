#![cfg(test)]

use pretty_assertions::assert_eq;
use syn::parse_quote;

use crate::{gen_read, gen_size, gen_struct, gen_write, PacketSyntax};

#[test]
fn test_structgen() {
    let packet_syntax: PacketSyntax = parse_quote! {
        #[derive(Debug)]
        #[test]
        pub struct PacketTest => server: &Server {
            init = {
                let player = server.player(uuid);
            }
            str(16) => let name: PacketString = player.name(),
            self => uuid: Uuid,
        }
    };
    let tokens = gen_struct(&packet_syntax);
    assert_eq!(
        "# [derive (Debug)] # [test] pub struct PacketTest { name : PacketString , uuid : Uuid } impl PacketTest { \
         pub fn new (server : & Server , uuid : Uuid) -> Self { let player = server . player (uuid) ; let name : \
         PacketString = player . name () ; Self { name , uuid } } }",
        tokens.to_string()
    );
}

#[test]
fn test_sizegen() {
    let packet_syntax: PacketSyntax = parse_quote! {
        #[derive(Debug)]
        #[test]
        pub struct PacketTest => server: &Server {
            init = {
                let player = server.player(uuid);
            }
            self as i32 => y: i16,
            var32 => z: i32,
            var64 => yz: i64,
            str(16) => let name: PacketString = player.name(),
            bytes => { test: PacketBytes, z = self.test.len() },
            self => uuid: Uuid,
            array => other: [i32; 5],
            bytearray => me: [u8; 10],
            nbt => x: MyStruct,
            rest => rest: PacketBytes,
        }
    };
    let tokens = gen_size(&packet_syntax);
    assert_eq!(
        "impl :: falcon_packet :: PacketSize for PacketTest { fn size (& self) -> usize { let z = self . test . len \
         () ; :: falcon_packet :: PacketSize :: size (& (self . y as i32)) + :: falcon_packet :: PacketSize :: size \
         (& < i32 as Into < :: falcon_packet :: primitives :: VarI32 >> :: into (z)) + :: falcon_packet :: PacketSize \
         :: size (& < i64 as Into < :: falcon_packet :: primitives :: VarI64 >> :: into (self . yz)) + :: \
         falcon_packet :: PacketSize :: size (< PacketString as AsRef < str >> :: as_ref (& self . name)) + :: \
         falcon_packet :: PacketSize :: size (& self . test) + :: falcon_packet :: PacketSize :: size (& self . uuid) \
         + :: falcon_packet :: PacketSize :: size (& self . other) + :: falcon_packet :: PacketSize :: size (& self . \
         me) + :: falcon_packet :: primitives :: nbt_size (& self . x) + :: falcon_packet :: PacketSize :: size (& \
         self . rest) } }",
        tokens.to_string()
    );
}

#[test]
fn test_readgen() {
    let packet_syntax: PacketSyntax = parse_quote! {
        #[derive(Debug)]
        #[test]
        pub struct PacketTest => server: &Server {
            init = {
                let player = server.player(uuid);
            }
            self as i32 => y: i16,
            var32 => z: i32,
            var64 => yz: i64,
            str(16) => let name: PacketString = player.name(),
            bytes => { test: PacketBytes, z = self.test.len() },
            self => uuid: Uuid,
            array => other: [i32; 5],
            bytearray => me: [u8; 10],
            nbt => x: MyStruct,
            rest => rest: PacketBytes,
        }
    };
    let tokens = gen_read(&packet_syntax);
    assert_eq!(
        "impl :: falcon_packet :: PacketRead for PacketTest { fn read < B > (buffer : & mut B) -> :: std :: result :: \
         Result < Self , :: falcon_packet :: ReadError > where B : :: bytes :: Buf + ? Sized , Self : Sized { let y = \
         < i32 as :: falcon_packet :: PacketRead > :: read (buffer) ? as i16 ; let z = < :: falcon_packet :: \
         primitives :: VarI32 as :: falcon_packet :: PacketRead > :: read (buffer) ? . into () ; let yz = < :: \
         falcon_packet :: primitives :: VarI64 as :: falcon_packet :: PacketRead > :: read (buffer) ? . into () ; let \
         name = :: falcon_packet :: PacketReadSeed :: read (16usize , buffer) ? ; let test = :: falcon_packet :: \
         PacketReadSeed :: read (self . z as usize , buffer) ? ; let uuid = :: falcon_packet :: PacketRead :: read \
         (buffer) ? ; let other = :: falcon_packet :: primitives :: array_read (buffer) ? ; let me = :: falcon_packet \
         :: primitives :: bytearray_read (buffer) ? ; let x = :: falcon_packet :: primitives :: nbt_read (buffer) ? ; \
         let rest = :: falcon_packet :: PacketReadSeed :: read (() , buffer) ? ; Ok (Self { y , z , yz , name , test \
         , uuid , other , me , x , rest }) } }",
        tokens.to_string()
    );
}

#[test]
fn test_writegen() {
    let packet_syntax: PacketSyntax = parse_quote! {
        #[derive(Debug)]
        #[test]
        pub struct PacketTest => server: &Server {
            init = {
                let player = server.player(uuid);
            }
            self as i32 => y: i16,
            var32 => z: i32,
            var64 => yz: i64,
            str(16) => let name: PacketString = player.name(),
            bytes => { test: PacketBytes, z = self.test.len() },
            self => uuid: Uuid,
            array => other: [i32; 5],
            bytearray => me: [u8; 10],
            nbt => x: MyStruct,
            rest => rest: PacketBytes,
        }
    };
    let tokens = gen_write(&packet_syntax);
    assert_eq!(
        "impl :: falcon_packet :: PacketWrite for PacketTest { fn write < B > (& self , buffer : & mut B) -> :: std \
         :: result :: Result < () , :: falcon_packet :: WriteError > where B : :: bytes :: BufMut { let z = self . \
         test . len () ; :: falcon_packet :: PacketWrite :: write (& (self . y as i32) , buffer) ? ; :: falcon_packet \
         :: PacketWrite :: write (& < i32 as Into < :: falcon_packet :: primitives :: VarI32 >> :: into (z) , buffer) \
         ? ; :: falcon_packet :: PacketWrite :: write (& < i64 as Into < :: falcon_packet :: primitives :: VarI64 >> \
         :: into (self . yz) , buffer) ? ; :: falcon_packet :: PacketWriteSeed :: write (16usize , < PacketString as \
         AsRef < str >> :: as_ref (& self . name) , buffer) ? ; :: falcon_packet :: PacketWrite :: write (< \
         PacketBytes as AsRef < [u8] >> :: as_ref (& self . test) , buffer) ? ; :: falcon_packet :: PacketWrite :: \
         write (& self . uuid , buffer) ? ; :: falcon_packet :: PacketWrite :: write (& self . other , buffer) ? ; :: \
         falcon_packet :: PacketWrite :: write (& self . me , buffer) ? ; :: falcon_packet :: primitives :: nbt_write \
         (& self . x , buffer) ? ; :: falcon_packet :: PacketWrite :: write (< PacketBytes as AsRef < [u8] >> :: \
         as_ref (& self . rest) , buffer) ? ; Ok (()) } }",
        tokens.to_string()
    );
}
