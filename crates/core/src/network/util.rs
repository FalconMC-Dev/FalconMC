pub fn read_var_i32_from_iter<I: Iterator<Item = u8>>(iterator: &mut I) -> Option<i32> {
    let mut result: i32 = 0;
    for i in 0..=6 {
        if i > 5 {
            return None;
        }
        let byte = match iterator.next() {
            None => return None,
            Some(x) => x,
        };
        result |= ((byte & 0x7f) as i32) << (i * 7);
        if byte & 0x80 == 0 {
            break;
        }
    }
    Some(result)
}
