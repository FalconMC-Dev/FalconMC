#![macro_use]

macro_rules! impl_packet_primitive_self {
    ( $type:ty, $fun_enc:ident, $fun_dec:ident ) => {
        impl_packet_encode_primitive_self!($type, $fun_enc);
        impl_packet_decode_primitive_self!($type, $fun_dec);
    };
}

macro_rules! impl_packet_encode_primitive_self {
    ( $type:ty, $fun:ident ) => {
        impl $crate::network::packet::PacketEncode for $type {
            fn to_buf(&self, buf: &mut dyn $crate::network::buffer::PacketBufferWrite) {
                buf.$fun(*self);
            }
        }
    };
}

macro_rules! impl_packet_decode_primitive_self {
    ( $type:ty, $fun:ident ) => {
        impl $crate::network::packet::PacketDecode for $type {
            fn from_buf(buf: &mut dyn $crate::network::buffer::PacketBufferRead) -> Result<Self> {
                buf.$fun()
            }
        }
    };
}
