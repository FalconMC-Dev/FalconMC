use bytes::Bytes;
use falcon_packet::packet;
use falcon_protocol::packets;

packet! {
    pub packet struct ExamplePacket {
        self => x: i32,
    }
}

packet! {
    pub packet struct PacketTwo {
        self => x: i32,
    }
}

packets! {
    ExamplePacket: { 47 = 0x01 },
    PacketTwo: { 47 = 0x03; 48 = 0x02 },
}

#[test]
fn test_switching() {
    let mut buffer = Bytes::from_static(&[0, 0, 0, 1]);
    let packet = read_packet(&mut buffer, 1, 48).unwrap();
    assert!(packet.is_none());
    let packet = read_packet(&mut buffer, 1, 47).unwrap();
    assert!(packet.is_some());
    let packet = packet.unwrap();
    assert!(packet.is::<ExamplePacket>());
    assert_eq!(1, packet.downcast_ref::<ExamplePacket>().unwrap().x);
}
