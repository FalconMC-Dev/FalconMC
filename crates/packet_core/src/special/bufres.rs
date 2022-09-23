use bytes::{BufMut, BytesMut};

pub trait BufRes: BufMut {
    fn reserve(&mut self, additional: usize);
}

impl BufRes for BytesMut {
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
