use bytes::{Buf, BufMut};

use crate::error::{ReadError, WriteError};
use crate::{PacketRead, PacketWrite};

impl PacketRead for bool {
    #[inline(always)]
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized,
    {
        Ok(buffer.get_u8() != 0)
    }
}

impl PacketWrite for bool {
    #[inline(always)]
    fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        buffer.put_u8(self as u8);
        Ok(())
    }
}

macro_rules! impl_num {
    ($($num:ident, $get:ident, $put:ident);*$(;)?) => {$(
        impl PacketRead for $num {
            #[inline(always)]
            fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
            where
                B: Buf + ?Sized,
                Self: Sized
            {
                Ok(buffer.$get())
            }
        }

        impl PacketWrite for $num {
            #[inline(always)]
            fn write<B>(self, buffer: &mut B) -> Result<(), WriteError>
            where
                B: BufMut + ?Sized
            {
                Ok(buffer.$put(self))
            }
        }
    )*}
}

impl_num! {
    i8, get_i8, put_i8;
    u8, get_u8, put_u8;
    i16, get_i16, put_i16;
    u16, get_u16, put_u16;
    i32, get_i32, put_i32;
    u32, get_u32, put_u32;
    i64, get_i64, put_i64;
    u64, get_u64, put_u64;
    f32, get_f32, put_f32;
    f64, get_f64, put_f64;
}
