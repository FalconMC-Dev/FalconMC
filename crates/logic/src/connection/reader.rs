use std::{mem::size_of, ptr};

use bytes::{buf::UninitSlice, Buf, BufMut, Bytes, BytesMut};
use flate2::{Decompress, FlushDecompress, Status};

const COMPRESSION_BUF_LEN: usize = 4096;

#[derive(Debug)]
pub struct SocketRead {
    decompress_buf: [u8; COMPRESSION_BUF_LEN],
    decompress: Decompress,
    decompress_pos: usize,
    compression_treshold: i32,
    output_buf: BytesMut,
    next_is_compressed: bool,
    ready_pos: usize,
    next_expected: usize,
}

impl SocketRead {
    pub fn new(compression_treshold: i32) -> Self {
        Self {
            decompress_buf: [0u8; COMPRESSION_BUF_LEN],
            decompress: Decompress::new(true),
            decompress_pos: 0,
            compression_treshold,
            output_buf: BytesMut::with_capacity(COMPRESSION_BUF_LEN),
            next_is_compressed: false,
            ready_pos: 0,
            next_expected: 0,
        }
    }

    pub fn next_packet(&mut self) -> Option<Bytes> {
        if self.decompress_pos > 0 {
            match self.flush_buffer() {
                Some(n) => self.decompress_pos = n,
                None => self.decompress_pos = 0,
            }
        }

        if self.ready_pos != 0 {
            let len = self.output_buf.get_i32();
            let data = self.output_buf.split_to(len as usize);
            self.ready_pos -= size_of::<i32>() + len as usize;
            return Some(data.freeze());
        }
        None
    }

    fn flush_buffer(&mut self) -> Option<usize> {
        let mut success = true;
        let mut start = 0;
        while start != self.decompress_pos && success {
            if self.next_expected == 0 {
                if let Some((length, cnt)) = read_varint_size(&self.decompress_buf[start..self.decompress_pos]) {
                    start += cnt;
                    if self.compression_treshold >= 0 {
                        if let Some((uncomp_len, cnt)) = read_varint_size(&self.decompress_buf[start..self.decompress_pos]) {
                            self.next_expected = length as usize - cnt;
                            if uncomp_len == 0 {
                                self.next_is_compressed = false;
                                self.output_buf.reserve(length as usize + size_of::<i32>());
                                self.output_buf.put_i32(length);
                            } else {
                                self.next_is_compressed = true;
                                self.output_buf.reserve(uncomp_len as usize + size_of::<i32>());
                                self.output_buf.put_i32(uncomp_len);
                            }
                            start += cnt;
                            start += self.read_next(start)?;
                        } else {
                            start -= cnt;
                            success = false;
                        }
                    } else {
                        self.next_is_compressed = false;
                        self.next_expected = length as usize;
                        self.output_buf.reserve(length as usize + size_of::<i32>());
                        self.output_buf.put_i32(length);
                        start += self.read_next(start)?;
                    }
                } else {
                    success = false;
                }
            } else {
                start += self.read_next(start)?;
            }
            if success && self.next_expected == 0 {
                if self.next_is_compressed {
                    self.decompress.reset(true);
                }
                self.ready_pos = self.output_buf.len() + size_of::<i32>();

                if self.output_buf.len() < COMPRESSION_BUF_LEN {
                    let capacity = self.output_buf.capacity();
                    if capacity > COMPRESSION_BUF_LEN && capacity > 3 * self.output_buf.len() {
                        let new_buffer = BytesMut::with_capacity(COMPRESSION_BUF_LEN);
                        let old_buffer = std::mem::replace(&mut self.output_buf, new_buffer);
                        self.output_buf.put(old_buffer);
                    }
                }
            }
        }
        let remaining = self.decompress_pos - start;
        if !success {
            if remaining <= start {
                // TODO: explain unsafe
                unsafe {
                    ptr::copy_nonoverlapping(
                        self.decompress_buf.as_ptr().add(start),
                        self.decompress_buf.as_mut_ptr(),
                        remaining,
                    );
                }
            } else {
                // TODO: explain unsafe
                unsafe {
                    ptr::copy(
                        self.decompress_buf.as_ptr().add(start),
                        self.decompress_buf.as_mut_ptr(),
                        remaining,
                    )
                }
            }
        }
        Some(remaining)
    }

    fn read_next(&mut self, mut start: usize) -> Option<usize> {
        let end = self.decompress_pos.min(start + self.next_expected);
        let len = end - start;
        if self.next_is_compressed {
            loop {
                let n = self.read_compressed(start, end)?;
                start += n;
                if n == 0 {
                    break;
                }
            }
        } else {
            let buf = &self.decompress_buf[start..end];
            self.output_buf.put_slice(buf);
        }
        self.next_expected -= len;
        Some(len)
    }

    fn read_compressed(&mut self, start: usize, end: usize) -> Option<usize> {
        loop {
            let input = &self.decompress_buf[start..end];
            let eof = input.is_empty();
            let before_out = self.decompress.total_out();
            let before_in = self.decompress.total_in();
            let flush = if eof {
                FlushDecompress::Finish
            } else {
                FlushDecompress::None
            };
            let ret = self.decompress.decompress(input, Self::output_mut(&mut self.output_buf), flush);
            let read = (self.decompress.total_out() - before_out) as usize;
            let consumed = (self.decompress.total_in() - before_in) as usize;
            unsafe {
                self.output_buf.advance_mut(read);
            }

            match ret {
                Ok(Status::Ok) | Ok(Status::BufError) if read == 0 && !eof => continue,
                Ok(Status::Ok) | Ok(Status::BufError) | Ok(Status::StreamEnd) => return Some(consumed),
                Err(_) => return None,
            }
        }
    }
    
    fn output_mut(buffer: &mut BytesMut) -> &mut [u8] {
        // TODO: explain unsafe
        unsafe {
            let chunk = buffer.chunk_mut();
            // This probably runs UB risk because of uninitialized memory
            // But because this is only used for writing and
            // not for reading, it seems to be working correctly so far.
            std::slice::from_raw_parts_mut(chunk.as_mut_ptr(), chunk.len())
        }
    }
}

fn read_varint_size(buf: &[u8]) -> Option<(i32, usize)> {
    let mut result = 0i32;
    // packet lengths can only be 3 bytes
    for (i, byte) in buf.iter().enumerate().take(3.min(buf.len()) + 1) {
        if i > 2 {
            return Some((result, i));
        }
        result |= ((byte & 0x7f) as i32) << (i * 7);
        if byte & 0x80 == 0 {
            return Some((result, i + 1));
        }
    }
    None
}

// TODO: explain unsafe
unsafe impl BufMut for SocketRead {
    fn remaining_mut(&self) -> usize {
        isize::MAX as usize
    }

    // TODO: explain unsafe
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.decompress_pos += cnt;

        if self.decompress_pos >= COMPRESSION_BUF_LEN {
            match self.flush_buffer() {
                Some(n) => self.decompress_pos = n,
                None => self.decompress_pos = 0,
            }
        }
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        // TODO: explain unsafe
        unsafe {
            UninitSlice::from_raw_parts_mut(
                self.decompress_buf.as_mut_ptr().add(self.decompress_pos),
                COMPRESSION_BUF_LEN - self.decompress_pos,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::BufMut;
    use itertools::Itertools;

    use super::SocketRead;

    /// Bad test, this needs some asserts
    #[test]
    pub fn test_read() {
        let mut reader = SocketRead::new(50);

        println!("Capacity: {}", reader.output_buf.capacity());

        reader.put_slice(&[0x8e, 0x00, 0xdc, 0x01, 0x78, 0x5e, 0x63, 0x64, 0x1c, 0xbe, 0x00, 0x00, 0x5f, 0xd2, 0x00, 0xdd]);
        // reader.put_slice(&[0x8d, 0x00, 0x00, 0x78, 0x5e, 0x63, 0x64, 0x1c, 0xbe, 0x00, 0x00, 0x5f, 0xd2, 0x00, 0xdd]);
        reader.put_slice(&[0x8d, 0x00]);
        // let res = reader.flush_buffer();

        let data = reader.next_packet();

        // println!("Res: {:?}", res);
        println!("Capacity: {}", reader.output_buf.capacity());
        println!("CompPos: {}", reader.decompress_pos);
        println!("Length: {}", reader.output_buf.len());
        println!("TotalIn: {}", reader.decompress.total_in());
        println!("ReadyPos: {}", reader.ready_pos);
        println!("Next_comp: {}", reader.next_is_compressed);
        println!("Next_exp: {}", reader.next_expected);
        println!("Content: {:02x}", reader.output_buf.as_ref().iter().format(" "));
        // println!("waiting: {:02x}", reader.decompress_buf[..res.unwrap()].as_ref().iter().format(" "));

        println!("Data: {:?}", data);


        // println!("Capacity: {}", reader.output_buf.capacity());
        // println!("CompPos: {}", reader.compression_position);
        // println!("Length: {}", reader.output_buf.len());
        // println!("ReadyPos: {}", reader.ready_pos);
        // println!("Content: {:02x}", reader.output_buf.as_ref().iter().format(" "));

        // let mut read = [0u8; 10];
        // reader.copy_to_slice(&mut read);

        // println!("Capacity: {}", reader.output_buf.capacity());
        // println!("CompPos: {}", reader.compression_position);
        // println!("Length: {}", reader.output_buf.len());
        // println!("ReadyPos: {}", reader.ready_pos);
        // println!("Content: {:02x}", reader.output_buf.as_ref().iter().format(" "));

        // reader.put_bytes(1, 110);
        // reader.finish();

        // println!("Capacity: {}", reader.output_buf.capacity());
        // println!("CompPos: {}", reader.compression_position);
        // println!("Length: {}", reader.output_buf.len());
        // println!("ReadyPos: {}", reader.ready_pos);
        // println!("Content: {:02x}", reader.output_buf.as_ref().iter().format(" "));
    }
}
