use bytes::{Buf, BufMut};

use crate::{PacketRead, ReadError, PacketWrite, WriteError, PacketSize};
use crate::primitives::{VarI32, VarI64};

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
                B: BufMut,
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
                ((({ $num::BITS - self.leading_zeros() } as usize).saturating_sub(1)) / 7) + 1
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
                    if i == var_max::<{ $num::BITS }>() {
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
    use bytes::{BytesMut, Bytes};

    use super::*;

    #[test]
    fn test_size() {
        let num = VarI32::from(3);
        assert_eq!(1, num.size());
        let num = VarI32::from(0b1_1111_1111_1111);
        assert_eq!(2, num.size());
        let num = VarI32::from(200);
        assert_eq!(2, num.size());
        let num = VarI32::from(4000);
        assert_eq!(2, num.size());
        let num = VarI32::from(0b1111_1111_1111_1111);
        assert_eq!(3, num.size());
    }

    #[test]
    fn test_write() {
        let mut buffer = BytesMut::new().limit(2);
        assert!(VarI32::from(1).write(&mut buffer).is_ok());
        assert!(VarI32::from(0b1111_1111).write(&mut buffer).is_err());

        let mut buffer = BytesMut::new();
        VarI32::from(-1).write(&mut buffer).unwrap();
        assert_eq!(&[0xff, 0xff, 0xff, 0xff, 0x0f], buffer.as_ref());
        buffer.clear();
        VarI32::from(-2147483648).write(&mut buffer).unwrap();
        assert_eq!(&[0x80, 0x80, 0x80, 0x80, 0x08], buffer.as_ref());
        buffer.clear();
        VarI32::from(128).write(&mut buffer).unwrap();
        VarI32::from(255).write(&mut buffer).unwrap();
        VarI32::from(2).write(&mut buffer).unwrap();
        VarI32::from(127).write(&mut buffer).unwrap();
        assert_eq!([0x80, 0x01, 0xff, 0x01, 0x02, 0x7f], buffer.as_ref());
    }

    #[test]
    fn test_read() {
        let mut buffer = Bytes::from_static(&[0xff, 0xff, 0xff, 0xff]);
        assert!(VarI32::read(&mut buffer).is_err());
        let mut buffer = Bytes::from_static(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        assert!(VarI32::read(&mut buffer).is_err());
        let mut buffer = Bytes::from_static(&[0xff, 0xff, 0xff, 0xff, 0x7f]);
        assert_eq!(-1, VarI32::read(&mut buffer).unwrap().val());
    }
}

