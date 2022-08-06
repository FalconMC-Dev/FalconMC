use std::io::Cursor;

use bytes::{Bytes, Buf};
use falcon_core::error::FalconCoreError;
use falcon_core::network::buffer::{PacketBufferRead, ByteLimitCheck};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct FalconCodec;

impl Decoder for FalconCodec {
    type Item = (i32, Bytes);

    type Error = FalconCoreError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buf = Cursor::new(src);
        let mut length: [u8; 3] = [0; 3];
        for i in 0..3 {
            if buf.remaining() == 0 {
                return Ok(None);
            }
            length[i] = buf.get_u8();
            if length[i] & 0b1000_0000 == 0 {
                let mut length = Cursor::new(&length[..]);
                let length = length.read_var_i32()? as usize;
                let src = buf.into_inner();
                if src.ensure_bytes_available(length).is_ok() {
                    let mut packet = src.split_to(i + 1 + length).split_off(i + 1).freeze();
                    let packet_id = packet.read_var_i32()?;
                    return Ok(Some((packet_id, packet)));
                } else {
                    src.reserve((i + 1 + length) - src.len());
                    return Ok(None);
                }
            }
        }
        Err(FalconCoreError::PacketTooLong)
    }
}

impl Encoder<()> for FalconCodec {
    type Error = FalconCoreError;

    fn encode(&mut self, _item: (), _dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }

}

