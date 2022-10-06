use bytes::{BufMut, BytesMut};

/// A buffer type that supports writing packet data to. This generally means
/// that this type automatically compresses data that is written to it and
/// prefixes that data with its length according to the minecraft protocol.
///
/// Because of this, **care** should be taken when using this type. The
/// [`prepare`](PacketPrepare::prepare) function must be called before the very
/// first byte of a packet is written. The [`finish`](PacketPrepare::finish)
/// function must be called after the last byte of a packet is written. If this
/// contract is not fulfilled, this type cannot guarantee a correct packet
/// stream.
///
/// ## **API usage**
/// This type should only be implemented by `FalconMC`.
pub trait PacketPrepare: BufMut {
    /// Prepares the buffer for the next packet to be written, given its length.
    ///
    /// # **Important**
    /// Only exact or upper limits of the packet's length will guarantee correct
    /// behavior.
    fn prepare(&mut self, length: usize);

    /// Called to signal that a packet has been written fully to this type. This
    /// allows the type to finish up the packet, for example prefixing it
    /// with the correct length.
    fn finish(&mut self);
}

/// Just writes data without any processing.
impl PacketPrepare for BytesMut {
    /// Any length is fine since `BytesMut` will indefinitely grow when
    /// required.
    fn prepare(&mut self, additional: usize) { self.reserve(additional); }

    /// No-op
    fn finish(&mut self) {}
}
