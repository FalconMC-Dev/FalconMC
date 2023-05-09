use bytes::{BufMut, Bytes, BytesMut};
use falcon_packet::primitives::PacketString;
use falcon_packet::{packet, size, write, PacketRead, PacketSize, PacketWrite, WriteError};

packet! {
    pub packet struct HandshakePacket {
        var32 => protocol: i32,
        str(255) => server_address: PacketString,
        self => port: u16,
        var32 => next_state: i32,
    }
}

#[test]
fn test_handshake_read() {
    let mut buffer = Bytes::from_static(&[0x2f, 0, 0x63, 0xdd, 0x02]);
    let packet = HandshakePacket::read(&mut buffer).unwrap();
    assert_eq!(47, packet.protocol);
    assert_eq!("", packet.server_address.as_ref());
    assert_eq!(25565, packet.port);
    assert_eq!(2, packet.next_state);
}

#[test]
fn test_handshake_write() {
    let mut buffer = BytesMut::new();
    let packet = HandshakePacket::new(47, String::from("").into(), 25565, 2);
    packet.write(&mut buffer).unwrap();
    assert_eq!(&[0x2f, 0, 0x63, 0xdd, 0x02], buffer.as_ref());
}

packet! {
    pub struct StructExample {
        num: i32,
        let plus_five: i32 = num + 5,
    }
}

impl PacketWrite for StructExample {
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        write! {
            var32 => i32 = self.num,
            self => i32 = self.num + 5,
        }
        Ok(())
    }
}
impl PacketSize for StructExample {
    fn size(&self) -> usize {
        size!(
            var32 => i32 = self.num,
            self => i32 = self.plus_five,
        )
    }
}

#[test]
fn test_impl() {
    let example = StructExample::new(10);
    assert_eq!(5, example.size());
}
