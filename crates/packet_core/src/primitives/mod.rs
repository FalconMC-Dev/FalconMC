mod array;
mod bytes;
mod iter;
mod num;
mod str;
mod vec;

#[cfg(test)]
mod tests;

pub use self::array::PacketArray;
pub use self::bytes::{AsRefU8, Bytes};
pub use self::iter::PacketIter;
pub use self::str::PacketString;
pub use self::vec::PacketVec;

macro_rules! impl_var_int {
    ($($var:ident: $base:ident => $($in:ident),+ + $($out_ty:ident),+);*$(;)?) => {$(
        #[doc = "Transparent wrapper around an "]
        #[doc = stringify!($base)]
        #[doc = "."]
        ///
        /// When writing to or reading from a minecraft connection, this will read itself using the
        /// [varint](https://wiki.vg/Protocol#VarInt_and_VarLong) representation.
        #[repr(transparent)]
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
            /// Return the stored value.
            pub fn val(self) -> $base {
                self.val
            }
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
    VarI32: i32 => i8, u8, i16, u16, isize, usize, u32 + isize, usize, u32, i64, u64, i128, u128;
    VarI64: i64 => i8, u8, i16, u16, i32, u32, isize, usize, u64 + u64, i128, u128;
}
