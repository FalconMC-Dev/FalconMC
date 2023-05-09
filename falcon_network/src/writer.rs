use std::io::{self, Write};

use bytes::buf::{UninitSlice, Writer};
use bytes::{Buf, BufMut, BytesMut};
use falcon_packet::primitives::VarI32;
use falcon_packet::{PacketSize, PacketWrite};
use flate2::write::ZlibEncoder;
use flate2::Compression;

const BUF_LEN: usize = 2usize.pow(12); // 2^12 = 4 KiB
const MAX_PACKET_LEN: usize = 2usize.pow(21) - 1; // maximum value of a 3 byte VarI32

/// A minecraft protocol packet writer.
///
/// A writer capable of correctly framing packets
/// for sending over the network. Deals with
/// compression and encryption when enabled.
/// This buffer implements [`BufMut`] for writing
/// and [`Buf`] for reading.
pub struct McWriter {
    input: [u8; BUF_LEN],
    input_len: usize,
    processed: CompressionBuffer,
    processed_pos: usize,
    next_length: Option<WriteLength>,
    compression: Option<i32>,
}

impl McWriter {
    pub fn new(compression: Option<i32>) -> Self {
        Self {
            input: [0u8; BUF_LEN],
            input_len: 0,
            processed: Default::default(),
            processed_pos: 0,
            next_length: None,
            compression,
        }
    }

    pub fn compression(&mut self, threshold: Option<i32>) { self.compression = threshold; }

    pub fn start_packet(&mut self, len: usize) -> io::Result<()> {
        if len > MAX_PACKET_LEN {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        if let Some(threshold) = self.compression {
            self.processed.bytes_mut().get_mut().reserve(len + 6);
            if threshold <= len as i32 {
                self.processed.compress_next();
                // Packet Length + Data Length
                self.processed
                    .bytes_mut()
                    .get_mut()
                    .put_slice(&[0x80, 0x80, 0x00, 0x80, 0x80, 0x00]);
                self.next_length = Some(WriteLength::Two)
            } else {
                // Packet Length + 1 zero
                self.processed.bytes_mut().get_mut().put_slice(&[0x80, 0x80, 0x00, 0x00]);
                self.next_length = Some(WriteLength::One)
            }
        } else {
            self.processed.bytes_mut().get_mut().reserve(len + 3);
            self.processed.bytes_mut().get_mut().put_slice(&[0x80, 0x80, 0x00]);
            self.next_length = Some(WriteLength::One)
        }
        Ok(())
    }

    // TODO: encryption
    pub fn finish_packet(&mut self) {
        if let Some(length) = self.next_length.take() {
            self.flush();
            let total_in = self.processed.finish().unwrap();
            let new_len = self.processed.bytes().len();
            let packet_len = new_len - self.processed_pos;
            let packet_buf = self.processed.bytes_mut().get_mut();
            match length {
                WriteLength::One => write_var32_unchecked(
                    (packet_len - 3) as i32,
                    &mut packet_buf[self.processed_pos..self.processed_pos + 3],
                ),
                WriteLength::Two => {
                    write_var32_unchecked(
                        (packet_len - 3) as i32,
                        &mut packet_buf[self.processed_pos..self.processed_pos + 3],
                    );
                    write_var32_unchecked(
                        total_in as i32,
                        &mut packet_buf[self.processed_pos + 3..self.processed_pos + 6],
                    )
                },
            }
            self.processed_pos = new_len;
        }
    }

    // TODO: resize output within bounds
    fn flush(&mut self) {
        if self.input_len > 0 {
            let buf = &self.input[..self.input_len];
            self.processed.write_all(buf).unwrap();
            self.input_len = 0;
        }
    }

    #[cfg(test)]
    fn is_input_empty(&self) -> bool { self.input_len == 0 }

    #[cfg(test)]
    fn is_output_empty(&self) -> bool { self.processed_pos == 0 }
}

// SAFETY: only `chunk_mut()` uses unsafe
unsafe impl BufMut for McWriter {
    fn remaining_mut(&self) -> usize { self.processed.bytes().remaining_mut() }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.input_len += cnt;
        if self.input_len >= BUF_LEN {
            self.flush();
        }
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        // SAFETY: this is the same way [`BytesMut`] works.
        unsafe {
            UninitSlice::from_raw_parts_mut(self.input.as_mut_ptr().add(self.input_len), BUF_LEN - self.input_len)
        }
    }
}

impl Buf for McWriter {
    fn remaining(&self) -> usize { self.processed_pos.min(self.processed.bytes().remaining()) }

    fn chunk(&self) -> &[u8] { &self.processed.bytes()[..self.remaining()] }

    fn advance(&mut self, cnt: usize) {
        self.processed.bytes_mut().get_mut().advance(cnt);
        self.processed_pos -= cnt;
    }
}

fn write_var32_unchecked(value: i32, buf: &mut [u8]) {
    let value = VarI32::from(value);
    let size = value.size();
    value.write(&mut &mut buf[..]).unwrap();
    if size < 3 {
        buf[size - 1] |= 0x80
    }
}

enum WriteLength {
    One,
    Two,
}

/// A buffer that transparently applies
/// compression when necessary. Behaves like [`Write`].
struct CompressionBuffer {
    buffer: ZlibEncoder<Writer<BytesMut>>,
    compressed: bool,
}

impl CompressionBuffer {
    /// Create a new buffer instance
    pub fn new() -> Self {
        Self {
            buffer: ZlibEncoder::new(BytesMut::with_capacity(BUF_LEN).writer(), Compression::new(7)),
            compressed: false,
        }
    }

    /// Marks the next packet as needing compression.
    ///
    /// This is not the same as enabling compression. When a
    /// packet's size is below the compression threshold, the
    /// packet is not actually compressed. The implementation
    /// must determine compression per packet.
    pub fn compress_next(&mut self) { self.compressed = true; }

    pub fn total_in(&self) -> usize { self.buffer.total_in() as usize }

    /// Finishes a packet and returns the amount of input bytes
    /// if compression was enabled or 0.
    ///
    /// If the packet was compressed, this method also resets
    /// the internal state of the decompressor. Packets
    /// that follow on this one shall not be decompressed (except
    /// see [`decompress_next`]).
    pub fn finish(&mut self) -> io::Result<usize> {
        let total_in = self.total_in();
        if self.compressed {
            self.compressed = false;
            self.buffer.try_finish()?;
            let writer = self.buffer.reset(BytesMut::new().writer())?;
            self.buffer.reset(writer)?;
        }
        Ok(total_in)
    }

    pub fn bytes(&self) -> &BytesMut { self.buffer.get_ref().get_ref() }

    /// Returns a mutable reference to the underlying [`Writer`].
    pub fn bytes_mut(&mut self) -> &mut Writer<BytesMut> { self.buffer.get_mut() }
}

impl Write for CompressionBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.compressed {
            self.buffer.write(buf)
        } else {
            self.buffer.get_mut().write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.compressed {
            self.buffer.flush()
        } else {
            self.buffer.get_mut().flush()
        }
    }
}

/// Compression is disabled by default.
impl Default for CompressionBuffer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use falcon_packet::PacketRead;

    use super::*;

    #[test]
    fn test_single_no_compression() {
        let mut writer = McWriter::new(None);
        writer.start_packet(6).unwrap();
        writer.put_bytes(2, 6);
        writer.finish_packet();
        let mut chunk = writer.copy_to_bytes(9);
        let length = VarI32::read(&mut chunk).unwrap().val();
        assert_eq!(6, length);
        assert_eq!(&[0x02, 0x02, 0x02, 0x02, 0x02, 0x02], &chunk[..]);
        assert!(writer.is_input_empty() && writer.is_output_empty());
    }

    #[test]
    fn test_multiple_no_compression() {
        let mut writer = McWriter::new(None);
        writer.start_packet(6).unwrap();
        writer.put_bytes(2, 6);
        writer.finish_packet();
        assert!(writer.is_input_empty());
        writer.start_packet(3).unwrap(); // protection against bad size estimates
        writer.put_slice(&[0x04, 0x05]);
        writer.finish_packet();
        assert!(writer.is_input_empty());
        // first packet
        let mut chunk = writer.copy_to_bytes(9);
        let length = VarI32::read(&mut chunk).unwrap().val();
        assert_eq!(6, length);
        assert_eq!(&[0x02, 0x02, 0x02, 0x02, 0x02, 0x02], &chunk[..]);
        // second packet
        let mut chunk = writer.copy_to_bytes(5);
        let length = VarI32::read(&mut chunk).unwrap().val();
        assert_eq!(2, length);
        assert_eq!(&[0x04, 0x05], &chunk[..]);
        assert!(writer.is_input_empty() && writer.is_output_empty());
    }

    #[test]
    fn test_too_long_no_compresion() {
        let mut writer = McWriter::new(None);
        let error = writer.start_packet(2usize.pow(22)).err().unwrap();
        assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    }

    #[test]
    fn test_single_compression_small() {
        let mut writer = McWriter::new(Some(1000));
        writer.start_packet(10).unwrap();
        writer.put_slice(&[1; 10]);
        writer.finish_packet();
        let len = writer.remaining();
        assert_eq!(14, len);
        let mut chunk = writer.copy_to_bytes(len);
        let stream_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(11, stream_len.val());
        let actual_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(0, actual_len.val());
        assert_eq!(&[1; 10], &chunk[..]);
    }

    #[test]
    fn test_single_compression_large() {
        let mut writer = McWriter::new(Some(1000));
        writer.start_packet(9000).unwrap();
        writer.put_slice(&[1; 9000]);
        writer.finish_packet();
        let len = writer.remaining();
        assert_eq!(38, len);
        let mut chunk = writer.copy_to_bytes(len);
        let stream_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(35, stream_len.val());
        let actual_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(9000, actual_len.val());
        assert_eq!(
            &[
                0x78, 0xda, 0xed, 0xc1, 0x01, 0x0d, 0x00, 0x00, 0x00, 0xc2, 0xa0, 0xbd, 0x7f, 0x69, 0x7b, 0x38, 0xa0,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0xdf, 0x00, 0x54, 0x12, 0x23, 0x29
            ],
            &chunk[..]
        );
    }

    #[test]
    fn test_multiple_compression_varied() {
        let mut writer = McWriter::new(Some(1000));
        writer.start_packet(9000).unwrap();
        writer.put_slice(&[1; 9000]);
        writer.finish_packet();
        assert_eq!(38, writer.remaining());
        writer.start_packet(10).unwrap();
        writer.put_slice(&[1; 10]);
        writer.finish_packet();
        assert_eq!(52, writer.remaining());
        let mut chunk = writer.copy_to_bytes(52);
        // first packet
        let stream_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(35, stream_len.val());
        let actual_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(9000, actual_len.val());
        assert_eq!(
            &[
                0x78, 0xda, 0xed, 0xc1, 0x01, 0x0d, 0x00, 0x00, 0x00, 0xc2, 0xa0, 0xbd, 0x7f, 0x69, 0x7b, 0x38, 0xa0,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0xdf, 0x00, 0x54, 0x12, 0x23, 0x29
            ],
            &chunk[..stream_len.as_usize() - 3]
        );
        // use the bytes
        chunk.advance(stream_len.as_usize() - 3);
        // second packet
        let stream_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(11, stream_len.val());
        let actual_len = VarI32::read(&mut chunk).unwrap();
        assert_eq!(0, actual_len.val());
        assert_eq!(&[1; 10], &chunk[..]);
    }
}
