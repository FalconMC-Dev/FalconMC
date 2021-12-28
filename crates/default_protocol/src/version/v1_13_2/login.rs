use falcon_core::network::buffer::PacketBufferRead;
use falcon_core::network::packet::PacketDecode;

use crate::errors::*;
use crate::implement_packet_handler_enum;
use crate::version::v1_8_9::login::LoginStartPacket;

pub enum LoginPackets {
    LoginStart(LoginStartPacket),
}

implement_packet_handler_enum!(LoginPackets, LoginStart);

impl LoginPackets {
    pub fn from_buf(packet_id: i32, buffer: &mut dyn PacketBufferRead) -> Result<Option<LoginPackets>> {
        match packet_id {
            0x00 => Ok(Some(LoginPackets::LoginStart(LoginStartPacket::from_buf(
                buffer,
            )?))),
            _ => Ok(None),
        }
    }
}
