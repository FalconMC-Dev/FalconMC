#![macro_use]

macro_rules! impl_packet_primitive_self {
    ( $type:ty, $fun_enc:ident, $fun_dec:ident ) => {
        impl_packet_encode_self!($type, $fun_enc);
        impl_packet_decode_self!($type, $fun_dec);
    };
}

macro_rules! impl_packet_encode_self {
    ( $type:ty, $fun:ident ) => {
        impl PacketEncode for $type {
            fn to_buf(self, buf: &mut dyn PacketBufferWrite) {
                buf.$fun(self);
            }
        }
    };
}

macro_rules! impl_packet_decode_self {
    ( $type:ty, $fun:ident ) => {
        impl PacketDecode for $type {
            fn from_buf(buf: &mut dyn PacketBufferRead) -> Result<Self> {
                Ok(buf.$fun()?)
            }
        }
    };
}
