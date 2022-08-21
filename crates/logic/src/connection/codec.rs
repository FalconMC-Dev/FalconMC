use std::io::Cursor;
use std::task::Poll;

use bytes::{Buf, Bytes, BytesMut};
use falcon_core::error::FalconCoreError;
use falcon_core::network::buffer::{ByteLimitCheck, PacketBufferRead};
use futures::{Future, ready};
use tokio::io::AsyncWrite;
use tokio::net::tcp::WriteHalf;
use tokio_util::codec::Decoder;
use tokio_util::io::poll_write_buf;

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

#[pin_project::pin_project]
pub(crate) struct TcpWrite<'a, 'b> {
    #[pin]
    pub(crate) socket: &'b mut WriteHalf<'a>,
    pub(crate) buffer: &'b mut BytesMut,
}

impl<'a, 'b> TcpWrite<'a, 'b> {
    pub(crate) fn new(socket: &'b mut WriteHalf<'a>, buffer: &'b mut BytesMut) -> Self {
        Self {
            socket,
            buffer,
        }
    }
}

impl<'a, 'b> Future for TcpWrite<'a, 'b> {
    type Output = Result<(), std::io::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut pinned = self.project();

        while !pinned.buffer.is_empty() {
            let n = ready!(poll_write_buf(pinned.socket.as_mut(), cx, pinned.buffer))?;
            if n == 0 {
                return Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to \
                     write frame to transport",
                )
                .into()));
            }
        }

        ready!(pinned.socket.poll_flush(cx))?;

        Poll::Ready(Ok(()))
    }
}



