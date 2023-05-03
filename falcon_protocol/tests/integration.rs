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
