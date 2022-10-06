use bytes::buf::UninitSlice;
use bytes::{Buf, BufMut, BytesMut};
use falcon_packet_core::special::PacketPrepare;
use falcon_packet_core::{PacketSize, VarI32};
use flate2::{Compress, Compression, FlushCompress, Status};

const COMPRESSION_BUFFER_LEN: usize = 4096;
/// See [the specification](https://www.rfc-editor.org/rfc/rfc1950#section-2).
/// This is header (= 2) + Adler checksum (= 4)
const ZLIB_EXTRA_LEN: usize = 6;

#[derive(Debug)]
pub struct SocketWrite {
    compression_buffer: [u8; COMPRESSION_BUFFER_LEN],
    compression_threshold: i32,
    compression: Compress,
    compression_position: usize,
    output_buffer: BytesMut,
    next_is_compressed: bool,
    next_len_size: usize,
    ready_pos: usize,
}

impl SocketWrite {
    pub fn new(threshold: i32) -> Self {
        Self {
            compression_buffer: [0; COMPRESSION_BUFFER_LEN],
            compression_threshold: threshold,
            compression: Compress::new(Compression::new(5), true),
            compression_position: 0,
            output_buffer: BytesMut::with_capacity(COMPRESSION_BUFFER_LEN),
            next_is_compressed: false,
            next_len_size: 0,
            ready_pos: 0,
        }
    }

    fn flush(&mut self) {
        self.write_all();
        self.compression_position = 0;

        if self.next_is_compressed {
            loop {
                let before = self.compression.total_out();
                self.compression
                    .compress(&[], Self::output_mut(&mut self.output_buffer), FlushCompress::Finish)
                    .unwrap();
                let n = self.compression.total_out();
                // TODO: explain unsafe
                unsafe { self.output_buffer.advance_mut((n - before) as usize) };

                if before == n {
                    return;
                }
            }
        }
    }

    fn write_all(&mut self) {
        if self.next_is_compressed {
            let mut n = 0;
            while !&self.compression_buffer[n..self.compression_position].is_empty() {
                n += self.write_compressed(n);
            }
        } else {
            let buf = &self.compression_buffer[..self.compression_position];
            self.output_buffer.put_slice(buf);
        }
    }

    fn write_compressed(&mut self, start: usize) -> usize {
        // flate2 isn't guaranteed to actually write any of the buffer provided,
        // it may be in a flushing mode where it's just giving us data before
        // we're actually giving it any data. We don't want to spuriously return
        // `Ok(0)` when possible as it will cause calls to write_all() to fail.
        // As a result we execute this in a loop to ensure that we try our
        // darndest to write the data.
        loop {
            let before_out = self.compression.total_out();
            let before_in = self.compression.total_in();
            let ret = self
                .compression
                .compress(&self.compression_buffer[start..self.compression_position], Self::output_mut(&mut self.output_buffer), FlushCompress::None)
                .unwrap();
            let written = (self.compression.total_in() - before_in) as usize;

            // TODO: explain unsafe
            unsafe { self.output_buffer.advance_mut((self.compression.total_out() - before_out) as usize) };

            let is_stream_end = matches!(ret, Status::StreamEnd);
            if !&self.compression_buffer[start..self.compression_position].is_empty() && written == 0 && !is_stream_end {
                continue;
            }

            return written;
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

fn write_fixed_varint(mut value: i32, size: usize, buf: &mut [u8]) {
    for byte in buf.iter_mut().take(size - 1) {
        *byte = (value & 127) as u8 | 128u8;
        value = ((value as u32) >> 7) as i32;
    }
    buf[size - 1] = value as u8;
}

impl PacketPrepare for SocketWrite {
    fn prepare(&mut self, additional: usize) {
        let len_size = VarI32::from(additional).size();
        let mut capacity = additional;
        if self.compression_threshold >= 0 {
            if self.compression_threshold <= additional as i32 {
                self.next_is_compressed = true;
                capacity += ZLIB_EXTRA_LEN;
                self.next_len_size = 3.min(VarI32::from(capacity + len_size).size()) + len_size;
            } else {
                self.next_is_compressed = false;
                self.next_len_size = 3.min(VarI32::from(capacity + 1).size()) + 1;
            }
        } else {
            self.next_is_compressed = false;
            self.next_len_size = len_size;
        }
        self.output_buffer.reserve(capacity + self.next_len_size);
        self.output_buffer.put_bytes(0, self.next_len_size);
    }

    fn finish(&mut self) {
        if self.ready_pos == self.output_buffer.len() {
            return;
        }

        self.flush();

        if self.compression_threshold >= 0 {
            if self.next_is_compressed {
                let offset = VarI32::from(self.compression.total_in() as usize).size();
                let overall_len = self.next_len_size - offset;
                write_fixed_varint((self.output_buffer.len() - self.ready_pos - overall_len) as i32, overall_len, &mut self.output_buffer[self.ready_pos..]);
                write_fixed_varint(self.compression.total_in() as i32, offset, &mut self.output_buffer[self.ready_pos + overall_len..]);
            } else {
                let overall_len = self.next_len_size - 1;
                write_fixed_varint((self.output_buffer.len() - self.ready_pos - overall_len) as i32, overall_len, &mut self.output_buffer[self.ready_pos..]);
            }
        } else {
            let overall_len = self.next_len_size;
            write_fixed_varint((self.output_buffer.len() - self.ready_pos - overall_len) as i32, self.next_len_size, &mut self.output_buffer[self.ready_pos..]);
        }

        // TODO: do encryption

        self.compression.reset();
        self.ready_pos = self.output_buffer.len();

        if self.output_buffer.len() < COMPRESSION_BUFFER_LEN {
            let capacity = self.output_buffer.capacity();
            if capacity > COMPRESSION_BUFFER_LEN && capacity > 3 * self.output_buffer.len() {
                let new_buffer = BytesMut::with_capacity(COMPRESSION_BUFFER_LEN);
                let old_buffer = std::mem::replace(&mut self.output_buffer, new_buffer);
                self.output_buffer.put(old_buffer);
            }
        }
    }
}

// TODO: explain unsafe code
unsafe impl BufMut for SocketWrite {
    fn remaining_mut(&self) -> usize { self.output_buffer.remaining_mut() }

    // TODO: explain unsafe
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.compression_position += cnt;

        if self.compression_position >= COMPRESSION_BUFFER_LEN {
            self.write_all();
            self.compression_position -= COMPRESSION_BUFFER_LEN;
        }
    }

    fn chunk_mut(&mut self) -> &mut UninitSlice {
        // TODO: explain unsafe
        unsafe {
            UninitSlice::from_raw_parts_mut(
                self.compression_buffer.as_mut_ptr().add(self.compression_position),
                COMPRESSION_BUFFER_LEN - self.compression_position,
            )
        }
    }
}

impl Buf for SocketWrite {
    fn remaining(&self) -> usize { self.ready_pos.min(self.output_buffer.remaining()) }

    fn chunk(&self) -> &[u8] { &self.output_buffer[..self.remaining()] }

    fn advance(&mut self, cnt: usize) {
        self.output_buffer.advance(cnt);
        self.ready_pos -= cnt;
    }
}

#[cfg(test)]
mod test {
    use bytes::{Buf, BufMut};
    use falcon_packet_core::special::PacketPrepare;
    use itertools::Itertools;

    use super::SocketWrite;

    /// Bad test, this needs some asserts
    #[test]
    pub fn test_write() {
        let mut writer = SocketWrite::new(50);

        println!("Capacity: {}", writer.output_buffer.capacity());

        writer.prepare(220);
        writer.put_bytes(1, 220);
        writer.finish();

        println!("Capacity: {}", writer.output_buffer.capacity());
        println!("CompPos: {}", writer.compression_position);
        println!("Length: {}", writer.output_buffer.len());
        println!("ReadyPos: {}", writer.ready_pos);
        println!("Content: {:02x}", writer.output_buffer.as_ref().iter().format(" "));

        writer.prepare(220);
        writer.put_bytes(1, 110);

        println!("Capacity: {}", writer.output_buffer.capacity());
        println!("CompPos: {}", writer.compression_position);
        println!("Length: {}", writer.output_buffer.len());
        println!("ReadyPos: {}", writer.ready_pos);
        println!("Content: {:02x}", writer.output_buffer.as_ref().iter().format(" "));

        let mut read = [0u8; 10];
        writer.copy_to_slice(&mut read);

        println!("Capacity: {}", writer.output_buffer.capacity());
        println!("CompPos: {}", writer.compression_position);
        println!("Length: {}", writer.output_buffer.len());
        println!("ReadyPos: {}", writer.ready_pos);
        println!("Content: {:02x}", writer.output_buffer.as_ref().iter().format(" "));

        writer.put_bytes(1, 110);
        writer.finish();

        println!("Capacity: {}", writer.output_buffer.capacity());
        println!("CompPos: {}", writer.compression_position);
        println!("Length: {}", writer.output_buffer.len());
        println!("ReadyPos: {}", writer.ready_pos);
        println!("Content: {:02x}", writer.output_buffer.as_ref().iter().format(" "));
    }
}
