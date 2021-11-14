use falcon_core::errors::*;
use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::connection::MinecraftConnection;
use falcon_core::network::packet::{PacketDecode, PacketHandler};

#[enum_dispatch::enum_dispatch(DispatchPacketHandler)]
pub enum LoginPackets {
    LoginStart(LoginStartPacket),
}

impl LoginPackets {
    pub fn from(packet_id: i32, buffer: &mut dyn PacketBufferRead) -> Result<Option<LoginPackets>> {
        match packet_id {
            0x00 => Ok(Some(LoginPackets::LoginStart(LoginStartPacket::from_buf(
                buffer,
            )?))),
            _ => Ok(None),
        }
    }
}

#[derive(PacketDecode)]
pub struct LoginStartPacket {
    name: String,
}

impl PacketHandler for LoginStartPacket {
    fn handle_packet(&mut self, _connection: &mut dyn MinecraftConnection) {
        debug!("Login start: {}", self.name);
    }

    fn get_name(&self) -> &'static str {
        "Login Start (1.8.9)"
    }
}
