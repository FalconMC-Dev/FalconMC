use bytes::{Buf, BufMut};

use crate::error::{ReadError, WriteError};
use crate::{PacketRead, PacketSize, PacketWrite, VarI32, VarI64};

impl PacketRead for bool {
    #[inline]
    fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
    where
        B: Buf + ?Sized,
        Self: Sized,
    {
        if !buffer.has_remaining() {
            return Err(ReadError::NoMoreBytes);
        }
        Ok(buffer.get_u8() != 0)
    }
}

impl PacketWrite for bool {
    #[inline]
    fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        if !buffer.has_remaining_mut() {
            return Err(WriteError::EndOfBuffer);
        }
        buffer.put_u8(*self as u8);
        Ok(())
    }
}

impl PacketSize for bool {
    #[inline]
    fn size(&self) -> usize { 1 }
}

macro_rules! impl_num {
    ($($num:ident, $get:ident, $put:ident);*$(;)?) => {$(
        impl PacketRead for $num {
            #[inline]
            fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
            where
                B: Buf + ?Sized,
                Self: Sized
            {
                if buffer.remaining() < ::std::mem::size_of::<$num>() {
                    return Err(ReadError::NoMoreBytes);
                }
                Ok(buffer.$get())
            }
        }

        impl PacketWrite for $num {
            #[inline]
            fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
            where
                B: BufMut + ?Sized
            {
                if buffer.remaining_mut() < ::std::mem::size_of::<$num>() {
                    return Err(WriteError::EndOfBuffer);
                }
                Ok(buffer.$put(*self))
            }
        }

        impl PacketSize for $num {
            #[inline]
            fn size(&self) -> usize {
                std::mem::size_of::<$num>()
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
    i128, get_i128, put_i128;
    u128, get_u128, put_u128;
    f32, get_f32, put_f32;
    f64, get_f64, put_f64;
}

const fn var_max<const BITS: u32>() -> usize { (BITS as usize + 6) / 7 }

macro_rules! impl_var {
    ($($var:ident = $num:ident & $unum:ident),*) => {$(
        impl PacketWrite for $var {
            fn write<B>(
                &self,
                buffer: &mut B,
            ) -> Result<(), WriteError>
            where
                B: BufMut + ?Sized,
            {
                let mut value = self.val;
                while value & -128 as $num != 0 {
                    ((value & 127 as $num) as u8 | 128u8).write(buffer)?;
                    value = ((value as $unum) >> 7) as $num;
                }
                (value as u8).write(buffer)
            }
        }

        impl PacketSize for $var {
            #[inline]
            fn size(&self) -> usize {
                ((({ $num::BITS - self.leading_zeros() } as usize).checked_sub(1).unwrap_or(0)) / 7) + 1
            }
        }

        impl PacketRead for $var {
            fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
            where
                B: Buf + ?Sized,
                Self: Sized
            {
                let mut result: $num = 0;
                for i in 0..=(var_max::<{ $num::BITS }>()) {
                    if i > var_max::<{ $num::BITS }>() {
                        return Err(ReadError::VarTooLong);
                    }
                    let byte = u8::read(buffer)?;
                    result |= ((byte & 0x7f) as $num) << (i * 7);
                    if byte & 0x80 == 0 {
                        break;
                    }
                }
                Ok($var::from(result))
            }
        }
    )*}
}

impl_var! { VarI32 = i32 & u32, VarI64 = i64 & u64 }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_size() {
        let num = VarI32::from(3);
        assert_eq!(num.size(), 1);
        let num = VarI32::from(0b1111111111111);
        assert_eq!(num.size(), 2);
        let num = VarI32::from(200);
        assert_eq!(num.size(), 2);
        let num = VarI32::from(4000);
        assert_eq!(num.size(), 2);
    }
}
