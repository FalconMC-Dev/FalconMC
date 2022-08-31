mod num {
    use crate::*;

    use bytes::Bytes;
    use core::fmt::Debug;

    #[test]
    fn test_num_read() {
        let zero = [0u8; 8]; // 0
        let small = [0u8, 0, 0, 0, 0, 0, 0, 0x7f]; // 127
        let medium = [0u8, 0, 0, 0, 0x07, 0x5B, 0xCD, 0x15]; // 123456789
        let large = [0u8, 0, 0x01, 0x1F, 0x71, 0xFB, 0x08, 0x43]; // 1234567891011

        test_buffers(
            [&zero, &small[7..], &medium[7..], &large[7..]],
            [0u8, 127u8, 21u8, 67u8],
        );
        test_buffers(
            [&zero, &small[7..], &medium[7..], &large[7..]],
            [0i8, 127i8, 21i8, 67i8],
        );
        test_buffers(
            [&zero, &small[6..], &medium[6..], &large[6..]],
            [0u16, 127u16, 52501u16, 2115u16],
        );
        test_buffers([&zero, &small[6..], &large[6..]], [0i16, 127i16, 2115i16]);
        test_buffers(
            [&zero, &small[4..], &medium[4..], &large[4..]],
            [0u32, 127u32, 123456789u32, 1912277059u32],
        );
        test_buffers(
            [&zero, &small[4..], &medium[4..], &large[4..]],
            [0i32, 127i32, 123456789i32, 1912277059i32],
        );
        test_buffers(
            [&zero, &small, &medium, &large],
            [0u64, 127u64, 123456789u64, 1234567891011u64],
        );
        test_buffers(
            [&zero, &small, &medium, &large],
            [0u64, 127u64, 123456789u64, 1234567891011u64],
        );
    }

    fn test_buffers<const N: usize, T>(buffers: [&[u8]; N], values: [T; N])
    where
        T: PacketRead + Debug + Eq,
    {
        for (i, buffer) in buffers.iter().enumerate() {
            assert_eq!(
                T::read(&mut Bytes::copy_from_slice(buffer)).unwrap(),
                values[i]
            );
        }
    }
}
