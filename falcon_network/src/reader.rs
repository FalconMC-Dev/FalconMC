use std::io::{self, Cursor, Write};
use std::ptr;

use aes::cipher::inout::InOutBuf;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use aes::Aes128;
use bytes::buf::{UninitSlice, Writer};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use cfb8::Decryptor;
use falcon_packet::primitives::VarI32;
use falcon_packet::{PacketRead, ReadError};
use flate2::write::ZlibDecoder;

const BUF_LEN: usize = 2usize.pow(12); // 2^12 = 4 KiB
const MAX_PACKET_LEN: usize = 2usize.pow(21) - 1; // maximum value of a 3 byte VarI32

/// A minecraft protocol packet reader.
///
/// A reader capable of extracting minecraft packets
/// from a minecraft network stream. Deals with
/// compression and encryption when enabled.
/// This buffer implements [`BufMut`].
///
/// # Example
/// ```
/// # use falcon_network::McReader;
/// # use bytes::BufMut;
/// // new reader without compression enabled
/// let mut reader = McReader::new(true, false);
///
/// // write a stream of packets, in this case one packet
/// reader.put_slice(&[0x06, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
///
/// // A full packet has been received, we expect a packet to be ready.
/// let packet = reader.next_packet().unwrap().unwrap();
/// assert_eq!(&[0x01, 0x02, 0x03, 0x04, 0x05], &packet[..]);
/// ```
///
/// # Single Packet Mode
/// When changing the state of this reader, it is not easy
/// to guarantee that packets that are still buffered are
/// read with the new state. To remedy this issue, the user can
/// enable Single Packet Mode temporarily so that automatic flushing
/// internally will be disabled. Only when a packet has been read
/// and returned can this buffer be flushed. Obviously this is
/// not intended to be enabled for long.
///
/// The most important thing to remember is that as long
/// as a packet is not returned, this buffer only has a capacity
/// of 4 KiB.
pub struct McReader {
    input: [u8; BUF_LEN],
    input_pos: usize,
    input_len: usize,
    next_expected: usize,
    needs_input: bool,
    processed: DecompressionBuffer,
    processed_pos: usize,
    compression: bool,
    decryptor: Option<Decryptor<Aes128>>,
    decryptor_pos: usize,
    is_single_packet: bool,
    is_corrupted: Option<io::Error>,
}

impl McReader {
    /// Create a new reader, compression can be enabled.
    /// If the creation of this reader happens at a time sensitive
    /// moment, it may be wise to enable single packet mode to avoid
    /// stream corruption.
    ///
    /// # Example
    /// ```
    /// # use falcon_network::McReader;
    /// # use bytes::BufMut;
    /// // new reader without compression enabled
    /// let mut reader = McReader::new(false, false);
    ///
    /// // input a two-byte packet
    /// reader.put_slice(&[0x02, 0x01, 0x02]);
    ///
    /// let packet = reader.next_packet().unwrap().unwrap();
    /// assert_eq!(&[0x01, 0x02], &packet[..]);
    /// ```
    pub fn new(compression: bool, is_single_packet: bool) -> Self {
        Self {
            input: [0; BUF_LEN],
            input_pos: 0,
            input_len: 0,
            next_expected: 0,
            needs_input: true,
            processed: Default::default(),
            processed_pos: 0,
            compression,
            decryptor: None,
            decryptor_pos: 0,
            is_single_packet,
            is_corrupted: None,
        }
    }

    /// Enables or disables compression. This will change
    /// the interpretation of input packets.
    ///
    /// # Example
    /// ```
    /// # use falcon_network::McReader;
    /// // create a new reader without compression
    /// let mut reader = McReader::new(false, false);
    ///
    /// // enable compression
    /// reader.compression(true);
    /// // disable compression
    /// reader.compression(false);
    /// ```
    pub fn compression(&mut self, enabled: bool) { self.compression = enabled; }

    /// Enables encryption.
    ///
    /// # Important
    /// This is a one time operation. ***It is not
    /// possible to disable encryption afterwards without
    /// risking corruption of the input stream.***
    pub fn encryption(&mut self, key: [u8; 16]) {
        // key cannot have invalid length -> safe unwrap
        self.decryptor = Some(Decryptor::new_from_slices(&key, &key).unwrap());
    }

    /// Enables or disables single packet mode.
    ///
    /// # Warning
    /// Care should be taken that no packets larger than at most 4 KiB
    /// are written to this reader. Failure to guarantee this will result
    /// in the reader seemingly having no more space remaining for more input.
    ///
    /// # Example
    /// ```
    /// # use falcon_network::McReader;
    /// // create a new reader without single packet mode
    /// let mut reader = McReader::new(false, false);
    ///
    /// // enable single packet mode
    /// reader.single_packet(true);
    /// // disable single packet mode
    /// reader.single_packet(false);
    /// ```
    pub fn single_packet(&mut self, is_single_packet: bool) { self.is_single_packet = is_single_packet; }

    /// Returns the next received packet or tries
    /// to flush the next packet in the buffer if no packets are ready.
    ///
    /// TODO: detail errors
    ///
    /// # Example
    /// ```
    /// # use falcon_network::McReader;
    /// # use bytes::BufMut;
    /// let mut reader = McReader::new(false, false);
    ///
    /// // write some data
    /// reader.put_slice(&[0x03, 0x01, 0x02, 0x03, 0x02, 0x00]);
    ///
    /// // get the next packet
    /// let packet = reader.next_packet().unwrap().unwrap();
    /// assert_eq!(&[0x01, 0x02, 0x03], &packet[..]);
    ///
    /// // we miss one byte to also get a second packet
    /// assert!(reader.next_packet().unwrap().is_none());
    /// // write the missing byte
    /// reader.put_u8(0x01);
    ///
    /// // read a second packet
    /// let packet = reader.next_packet().unwrap().unwrap();
    /// assert_eq!(&[0x00, 0x01], &packet[..]);
    /// ```
    pub fn next_packet(&mut self) -> io::Result<Option<Bytes>> {
        if let Some(err) = self.is_corrupted.take() {
            return Err(err);
        }
        if self.processed_pos > 0 {
            self.extract_packet()
        } else {
            self.flush()?;
            if self.processed_pos > 0 {
                self.extract_packet()
            } else {
                Ok(None)
            }
        }
    }

    fn extract_packet(&mut self) -> io::Result<Option<Bytes>> {
        let len = self.processed.bytes_mut().get_mut().get_u32() as usize;
        if self.processed_pos < len {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let bytes = self.processed.bytes_mut().get_mut().split_to(len);
        self.processed_pos -= bytes.len() + 4; // 4 is because i32
        Ok(Some(bytes.freeze()))
    }

    /// Flushes the input buffer and prepares the next packet.
    /// Preparing as many packets as possible will eliminate
    /// the ability to change the compression state and/or
    /// enable encryption. That's why this only happens when
    /// not in single packet mode.
    /// TODO: resize output buffer within bounds
    fn flush(&mut self) -> io::Result<()> {
        if self.is_corrupted.is_some() {
            return Ok(());
        }

        // decrypt if necessary
        if let Some(decryptor) = &mut self.decryptor {
            if self.decryptor_pos < self.input_len {
                let (blocks, _) = InOutBuf::from(&mut self.input[self.decryptor_pos..self.input_len]).into_chunks();
                decryptor.decrypt_blocks_inout_mut(blocks);
                self.decryptor_pos = self.input_len;
            }
        }

        while self.input_pos < self.input_len && !self.needs_input {
            if self.next_expected > 0 {
                // packet is still being read
                self.continue_read()?;
                if self.is_single_packet {
                    break;
                }
            } else {
                self.needs_input = self.start_new_packet()?;
            }
        }

        let diff = self.input_len - self.input_pos;
        if self.input_pos > diff {
            unsafe {
                ptr::copy_nonoverlapping(self.input.as_ptr().add(self.input_pos), self.input.as_mut_ptr(), diff);
            }
        } else {
            unsafe { ptr::copy(self.input.as_ptr().add(self.input_pos), self.input.as_mut_ptr(), diff) }
        }
        if self.decryptor.is_some() {
            self.decryptor_pos = diff;
        }
        self.input_len = diff;
        self.input_pos = 0;
        Ok(())
    }

    fn continue_read(&mut self) -> io::Result<()> {
        // Take up to `next_expected` bytes from input
        let buf = &self.input[self.input_pos..self.input_len.min(self.input_pos + self.next_expected)];
        let len = buf.len();
        self.processed.write_all(buf)?; // write `len` bytes to processed
        self.input_pos += len;
        self.next_expected -= len;
        if self.next_expected == 0 {
            // if packet is fully received
            self.processed_pos = self.processed.finish()?;
        }
        Ok(())
    }

    // TODO: Reserve space in output buffer
    fn start_new_packet(&mut self) -> io::Result<bool> {
        let mut cursor = Cursor::new(&self.input[self.input_pos..self.input_len]);
        match VarI32::read(&mut cursor) {
            Ok(stream_len) => {
                if stream_len.as_usize() > MAX_PACKET_LEN {
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
                }
                if self.is_single_packet
                    && (self.input_len - self.input_pos - cursor.position() as usize) < stream_len.as_usize()
                {
                    return Ok(true);
                }
                if self.compression {
                    let stream_pos = cursor.position() as usize;
                    match VarI32::read(&mut cursor) {
                        Ok(length) => {
                            if length.as_usize() > MAX_PACKET_LEN {
                                return Err(io::Error::from(io::ErrorKind::InvalidData));
                            }
                            if length.val() != 0 {
                                // write length for later reference
                                self.processed
                                    .bytes_mut()
                                    .write_all(&length.as_u32().to_be_bytes())
                                    .unwrap();
                                self.processed.decompress_next();
                            } else {
                                // write length for later reference
                                self.processed
                                    .bytes_mut()
                                    .write_all(&(stream_len.as_u32() - 1).to_be_bytes())
                                    .unwrap();
                            }
                            self.next_expected = stream_len.as_usize() - cursor.position() as usize + stream_pos;
                            self.input_pos += cursor.position() as usize;
                        },
                        Err(ReadError::NoMoreBytes) => return Ok(true),
                        Err(ReadError::VarTooLong) => return Err(io::Error::from(io::ErrorKind::InvalidData)),
                        Err(_) => {
                            unreachable!("The `VarI32` PacketRead implementation should only error on `NoMoreBytes`!")
                        },
                    }
                } else {
                    self.processed
                        .bytes_mut()
                        .write_all(&stream_len.as_u32().to_be_bytes())
                        .unwrap();
                    self.next_expected = stream_len.as_usize();
                    self.input_pos += cursor.position() as usize;
                }
            },
            Err(ReadError::NoMoreBytes) => return Ok(true),
            Err(ReadError::VarTooLong) => return Err(io::Error::from(io::ErrorKind::InvalidData)),
            Err(_) => unreachable!(
                "The `VarI32` PacketRead implementation should only error on `NoMoreBytes` or `VarTooLong`!"
            ),
        }
        Ok(false)
    }

    #[cfg(test)]
    fn input_is_empty(&self) -> bool { self.input_len == 0 }
}

// Only the `UninitSlice` operation is unsafe,
// but this is done in the same way [`BytesMut`] does it.
unsafe impl BufMut for McReader {
    fn remaining_mut(&self) -> usize {
        if self.is_corrupted.is_some() {
            0
        } else if self.is_single_packet {
            BUF_LEN - self.input_len
        } else {
            usize::MAX - self.input_len
        }
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        if self.is_corrupted.is_none() {
            self.needs_input = false;
            self.input_len += cnt;
            if self.input_len >= BUF_LEN && !self.is_single_packet {
                if let Err(e) = self.flush() {
                    self.is_corrupted = Some(e);
                }
            }
        }
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        // Safety: Same mechanics as [`BytesMut`].
        unsafe {
            UninitSlice::from_raw_parts_mut(self.input.as_mut_ptr().add(self.input_len), BUF_LEN - self.input_len)
        }
    }
}

/// A buffer that transparently applies
/// decompression when necessary. Behaves like [`Write`].
struct DecompressionBuffer {
    buffer: ZlibDecoder<Writer<BytesMut>>,
    compressed: bool,
}

impl DecompressionBuffer {
    /// Create a new buffer instance
    pub fn new() -> Self {
        Self {
            buffer: ZlibDecoder::new(BytesMut::with_capacity(BUF_LEN).writer()),
            compressed: false,
        }
    }

    /// Marks the next packet as needing decompression.
    ///
    /// This is not the same as enabling compression. When a
    /// packet's size is below the compression threshold, the
    /// packet is not actually compressed. The implementation
    /// must determine decompression per packet.
    pub fn decompress_next(&mut self) { self.compressed = true; }

    /// Finishes a packet and returns the ready position
    /// of the next packet.
    ///
    /// If the packet was compressed, this method also resets
    /// the internal state of the decompressor. Packets
    /// that follow on this one shall not be decompressed (except
    /// see [`decompress_next`]).
    pub fn finish(&mut self) -> io::Result<usize> {
        if self.compressed {
            self.compressed = false;
            self.buffer.try_finish()?;
            let writer = self.buffer.reset(BytesMut::new().writer())?;
            self.buffer.reset(writer)?;
        }
        Ok(self.buffer.get_ref().get_ref().len())
    }

    /// Returns a mutable reference to the underlying [`Writer`].
    pub fn bytes_mut(&mut self) -> &mut Writer<BytesMut> { self.buffer.get_mut() }
}

impl Write for DecompressionBuffer {
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
impl Default for DecompressionBuffer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use falcon_packet::PacketWrite;

    use super::*;

    #[test]
    fn test_single_no_compression() {
        let mut reader = McReader::new(false, false);
        reader.put_slice(&[0x05, 0x01, 0x0, 0x0, 0x0, 0x4]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x0, 0x0, 0x0, 0x04], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_multiple_no_compression() {
        let mut reader = McReader::new(false, false);
        reader.put_slice(&[0x05, 0x01, 0x0, 0x0, 0x0, 0x4, 0xA, 0x2, 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x0, 0x0, 0x0, 0x4], &packet[..]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x2, 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_too_long_no_compression() {
        let mut reader = McReader::new(false, false);
        reader.put_slice(&[0xff, 0xff, 0xff, 0xef, 0x0]);
        let error = reader.next_packet().err().unwrap();
        assert_eq!(io::ErrorKind::InvalidData, error.kind());
    }

    #[test]
    fn test_single_no_compression_separated() {
        let mut reader = McReader::new(false, false);
        reader.put_slice(&[0x0a, 0x01, 0x0, 0x0, 0x0, 0x4]);
        assert!(reader.next_packet().unwrap().is_none());
        reader.put_slice(&[0x0, 0x0, 0x0, 0x0]);
        assert!(reader.next_packet().unwrap().is_none());
        reader.put_u8(0x0);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x0, 0x0, 0x0, 0x04, 0x0, 0x0, 0x0, 0x0, 0x0], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_multiple_no_compression_separated() {
        let mut reader = McReader::new(false, false);
        reader.put_slice(&[0x0a, 0x01, 0x2, 0x3, 0x4, 0x5]);
        assert!(reader.next_packet().unwrap().is_none());
        reader.put_slice(&[0x6, 0x7, 0x8, 0x9]);
        assert!(reader.next_packet().unwrap().is_none());
        reader.put_slice(&[0xa, 0x06, 0x01]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        reader.put_slice(&[0x03, 0x02, 0x4, 0x5, 0x6]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x03, 0x02, 0x4, 0x5, 0x6], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_single_compression_small() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[0x06, 0x00, 0x01, 0x0, 0x0, 0x0, 0x4]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x0, 0x0, 0x0, 0x04], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_multiple_compression_small() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[
            0x6, 0x0, 0x1, 0x0, 0x0, 0x0, 0x4, 0xB, 0x0, 0x2, 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x0, 0x0, 0x0, 0x4], &packet[..]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x2, 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_single_compression_large() {
        let mut reader = McReader::new(true, false);
        // 100 `0x00` bytes zlib compressed
        reader.put_slice(&[0x0d, 0x64, 0x78, 0x5e, 0x63, 0x60, 0xa0, 0x3d, 0x00, 0x00, 0x00, 0x64, 0x00, 0x01]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(100, packet.len());
        let actual = [0; 100];
        assert_eq!(&actual[..], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_multiple_compression_large() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[
            0x0d, 0x64, 0x78, 0x5e, 0x63, 0x60, 0xa0, 0x3d, 0x00, 0x00, 0x00, 0x64, 0x00, 0x01, 0x0e, 0xc8, 0x01, 0x78,
            0x5e, 0x63, 0x64, 0x1c, 0x1e, 0x00, 0x00, 0x4f, 0x4c, 0x00, 0xc9,
        ]);
        // 100 zeros
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(100, packet.len());
        let actual = [0; 100];
        assert_eq!(&actual[..], &packet[..]);
        // 200 ones
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(200, packet.len());
        let actual = [1; 200];
        assert_eq!(&actual[..], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_multiple_compression_varied() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[
            0x06, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x0e, 0xc8, 0x01, 0x78, 0x5e, 0x63, 0x64, 0x1c, 0x1e, 0x00, 0x00,
            0x4f, 0x4c, 0x00, 0xc9,
        ]);
        // 5 bytes
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[1, 0, 0, 0, 4], &packet[..]);
        // 200 ones
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(200, packet.len());
        let actual = [1; 200];
        assert_eq!(&actual[..], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn too_too_long_stream_compression() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[0xff, 0xff, 0xff, 0xef, 0x0]);
        let error = reader.next_packet().err().unwrap();
        assert_eq!(io::ErrorKind::InvalidData, error.kind());
    }

    #[test]
    fn test_too_long_actual_compression() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[0x0a, 0xff, 0xff, 0xff, 0xef, 0x00]);
        let error = reader.next_packet().err().unwrap();
        assert_eq!(io::ErrorKind::InvalidData, error.kind());
    }

    #[test]
    fn test_single_compression_separated() {
        let mut reader = McReader::new(true, false);
        // 100 `0x00` bytes zlib compressed
        reader.put_slice(&[0x0d, 0x64, 0x78, 0x5e, 0x63, 0x60, 0xa0, 0x3d]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
        reader.put_slice(&[0x00, 0x00, 0x00, 0x64, 0x00, 0x01]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(100, packet.len());
        let actual = [0; 100];
        assert_eq!(&actual[..], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_multiple_compression_separated() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[0x0d, 0x64, 0x78, 0x5e, 0x63]);
        assert!(reader.next_packet().unwrap().is_none() && reader.input_is_empty());
        reader.put_slice(&[0x60, 0xa0, 0x3d, 0x00, 0x00, 0x00, 0x64, 0x00, 0x01, 0x0e]);
        // 100 zeros
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(100, packet.len());
        let actual = [0; 100];
        assert_eq!(&actual[..], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none() && !reader.input_is_empty());
        reader.put_slice(&[0xc8, 0x01, 0x78, 0x5e, 0x63, 0x64]);
        assert!(reader.next_packet().unwrap().is_none() && reader.input_is_empty());
        reader.put_slice(&[0x1c, 0x1e, 0x00, 0x00, 0x4f, 0x4c, 0x00, 0xc9]);
        // 200 ones
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(200, packet.len());
        let actual = [1; 200];
        assert_eq!(&actual[..], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        assert!(reader.input_is_empty());
    }

    #[test]
    fn test_invalid_varint_no_compression() {
        let mut reader = McReader::new(false, false);
        reader.put_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        let error = reader.next_packet().err().unwrap();
        assert_eq!(io::ErrorKind::InvalidData, error.kind());
    }

    #[test]
    fn test_invalid_varint_compression() {
        let mut reader = McReader::new(true, false);
        reader.put_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        let error = reader.next_packet().err().unwrap();
        assert_eq!(io::ErrorKind::InvalidData, error.kind());

        let mut reader = McReader::new(true, false);
        reader.put_slice(&[0x0a, 0xff, 0xff, 0xff, 0xff, 0xff]);
        let error = reader.next_packet().err().unwrap();
        assert_eq!(io::ErrorKind::InvalidData, error.kind());
    }

    #[test]
    fn really_large_input() {
        let mut reader = McReader::new(false, false);
        VarI32::from(9000).write(&mut reader).unwrap();
        reader.put_slice(&[1; 9000]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[1; 9000], &packet[..]);
    }

    #[test]
    #[should_panic]
    fn really_large_input_single_packet_mode() {
        let mut reader = McReader::new(false, true);
        VarI32::from(9000).write(&mut reader).unwrap();
        reader.put_slice(&[1; 9000]);
    }

    #[test]
    fn test_single_packet_mode_correct_usage() {
        let mut reader = McReader::new(false, true);
        reader.put_slice(&[0x02, 0x01, 0x03, 0x04, 0x00, 0x02, 0x03, 0x01]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x03], &packet[..]);
        reader.compression(true);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x02, 0x03, 0x01], &packet[..]);
    }

    #[test]
    fn test_single_packet_mode_correct_usage_separated() {
        let mut reader = McReader::new(false, true);
        reader.put_slice(&[0x02, 0x01, 0x03, 0x04, 0x00]);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x01, 0x03], &packet[..]);
        assert!(reader.next_packet().unwrap().is_none());
        reader.put_slice(&[0x02, 0x03, 0x01]);
        reader.compression(true);
        let packet = reader.next_packet().unwrap().unwrap();
        assert_eq!(&[0x02, 0x03, 0x01], &packet[..]);
    }
}
