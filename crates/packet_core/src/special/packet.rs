use bytes::{BufMut, BytesMut};

pub trait PacketPrepare: BufMut {
    fn prepare(&mut self, additional: usize);
}

impl PacketPrepare for BytesMut {
    fn prepare(&mut self, additional: usize) { self.reserve(additional); }
}
