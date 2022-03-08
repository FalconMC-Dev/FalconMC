mod read;
mod write;

pub use read::{ByteLimitCheck, PacketBufferRead};
pub use write::PacketBufferWrite;

/// Returns the amount of bytes needed to encode this i32 in [var-int](https://wiki.vg/Protocol#VarInt_and_VarLong) format.
pub fn get_var_i32_size(input: i32) -> usize {
    for i in 1..5 {
        if (input & (-1_i32 << (i * 7))) == 0 {
            return i;
        }
    }
    5
}

/// Returns the amount of bytes needed to encode this i64 in [var-int](https://wiki.vg/Protocol#VarInt_and_VarLong) format.
pub fn get_var_i64_size(input: i64) -> usize {
    for i in 1..10 {
        if (input & (-1_i64 << (i * 7))) == 0 {
            return i;
        }
    }
    10
}

pub fn as_u8_slice(v: &[i8]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len()) }
}

pub fn read_var_i32_from_iter<I: Iterator<Item=u8>>(iterator: &mut I) -> Option<i32> {
    let mut result: i32 = 0;
    for i in 0..=6 {
        if i > 5 {
            return None;
        }
        let byte = match iterator.next() {
            None => return None,
            Some(x) => x,
        };
        result |= ((byte & 0b0111_1111) as i32) << (i * 7);
        if byte & 0b1000_0000 == 0 {
            break;
        }
    }
    Some(result)
}
