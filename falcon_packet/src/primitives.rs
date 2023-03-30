//! Collection of types that have a predefined
//! implementation of the packet traits.
//!
//! Currently, all the types that are supported are:
//! - Primitive types:
//!     - bool
//!     - i8, i16, i32, i64
//!     - u8, u16, u32, u64
//!     - f32, f64
//! - Byte array types:
//!     - [`PacketBytes`]
//!     - [`Bytes`](::bytes::Bytes), [`BytesMut`](::bytes::BytesMut)
//!     - [`write_bytes`]
//! - String types:
//!     - [`&str`] and [`AsRef<str>`]
//!     - [`PacketString`]
//!     - [`write_str`], [`write_str_unchecked`]
//! - Uuid types:
//!     - [`Uuid`](::uuid::Uuid)
//!     - [`StringUuid`]
//! - Collection types:
//!     - [`Vec<T>`](Vec)
//!     - [`[T]`](slice)
//! - Iterators:
//!     - [`iter_write`], [`iter_size`]
//!     - [`iter_read`]

mod boolean;
mod bytes;
mod num;
mod slices;
mod string;
mod util;
mod uuid;
mod vec;

pub use util::*;

pub use self::bytes::*;
pub use self::string::*;
pub use self::uuid::*;
pub use self::vec::*;

macro_rules! impl_var_int {
    ($($var:ident: $base:ident => $($in:ident),+ + $($out_ty:ident = $out:ident),+);*$(;)?) => {$(
        #[doc = stringify!(A variable length wrapper for $base,)]
        #[doc = "see [here](https://wiki.vg/Protocol#VarInt_and_VarLong)."]
        #[repr(transparent)]
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Default)]
        pub struct $var {
            val: $base,
        }

        impl std::ops::Deref for $var {
            type Target = $base;

            fn deref(&self) -> &Self::Target {
                &self.val
            }
        }

        impl std::ops::DerefMut for $var {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.val
            }
        }

        impl $var {
            pub fn val(self) -> $base {
                self.val
            }

            $(pub fn $out(self) -> $out_ty {
                self.val as $out_ty
            })+
        }

        impl From<$base> for $var {
            fn from(val: $base) -> Self {
                Self {
                    val,
                }
            }
        }

        $(impl From<$in> for $var {
            fn from(val: $in) -> Self {
                Self {
                    val: val as $base,
                }
            }
        })+

        impl From<$var> for $base {
            fn from(val: $var) -> Self {
                val.val
            }
        }

        $(impl From<$var> for $out_ty {
            fn from(val: $var) -> Self {
                val.val as $out_ty
            }
        })+
    )*}
}

impl_var_int! {
    VarI32: i32 => i8, u8, i16, u16, isize, usize, u32 +
    isize = as_isize, usize = as_usize, u32 = as_u32, i64 = as_i64, u64 = as_u64, i128 = as_i128, u128 = as_u128;
    VarI64: i64 => i8, u8, i16, u16, i32, u32, isize, usize, u64 + u64 = as_u64, i128 = as_i128, u128 = as_u128;
}
