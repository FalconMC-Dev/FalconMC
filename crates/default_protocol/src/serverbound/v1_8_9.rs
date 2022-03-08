pub use inner::*;

#[falcon_default_protocol_derive::packet_module]
mod inner {
    use falcon_core::network::connection::MinecraftConnection;
    use falcon_core::network::packet::PacketEncode;

    pub fn handle_packet<R: ::falcon_core::network::buffer::PacketBufferRead, C: MinecraftConnection>(packet_id: i32, buffer: &mut R, connection: &mut C) {

    }

    #[derive(PacketEncode)]
    pub struct TestPacket {
        dummy: i32,
    }
}