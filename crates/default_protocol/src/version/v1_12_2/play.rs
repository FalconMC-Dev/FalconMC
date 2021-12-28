use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketEncode, PacketHandler};

#[derive(PacketEncode, PacketDecode, new)]
pub struct KeepAlivePacket {
    id: i64,
}

impl PacketHandler for KeepAlivePacket {
    fn handle_packet(self, connection: &mut dyn MinecraftConnection) {
        if connection.get_handler_state().last_keep_alive() != self.id as u64 {
            connection.disconnect(String::from("Received invalid Keep Alive id!"));
        } else {
            connection.reset_keep_alive();
        }
    }

    fn get_name(&self) -> &'static str {
        "Keep Alive Packet (1.12.2)"
    }
}