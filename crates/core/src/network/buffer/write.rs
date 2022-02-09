use bytes::BufMut;
use uuid::Uuid;

pub trait PacketBufferWrite {
    /// Writes an [`i8`] to the underlying buffer.
    fn write_i8(&mut self, value: i8);

    /// Writes an [`u8`] to the underlying buffer.
    fn write_u8(&mut self, value: u8);

    /// Writes an [`i16`] to the underlying buffer.
    fn write_i16(&mut self, value: i16);

    /// Writes an [`u16`] to the underlying buffer.
    fn write_u16(&mut self, value: u16);

    /// Writes an [`i32`] to the underlying buffer.
    fn write_i32(&mut self, value: i32);

    /// Writes an [`i64`] to the underlying buffer.
    fn write_i64(&mut self, value: i64);

    /// Writes an [`f32`] to the underlying buffer.
    fn write_f32(&mut self, value: f32);

    /// Writes an [`f64`] to the underlying buffer.
    fn write_f64(&mut self, value: f64);

    /// Writes a [`Uuid`] to the underlying buffer.
    fn write_uuid(&mut self, uuid: &Uuid);

    /// Writes a byte array to the underlying buffer.
    fn write_u8_array(&mut self, array: &[u8]);

    /// Writes a [`bool`] to the underlying buffer.
    fn write_bool(&mut self, value: bool) {
        self.write_u8(if value { 0x01 } else { 0x00 });
    }

    /// Writes a [`String`] to the underlying buffer.
    fn write_string(&mut self, string: &str) {
        self.write_var_i32(string.len() as i32);
        self.write_u8_array(string.as_bytes());
    }

    /// Writes an [`i32`] in [var-int](https://wiki.vg/Protocol#VarInt_and_VarLong) format to the underlying buffer.
    fn write_var_i32(&mut self, mut value: i32) {
        while value & -128i32 != 0 {
            self.write_u8(((value & 127i32) as u8) | 128u8);
            value = ((value as u32) >> 7) as i32;
        }
        self.write_u8(value as u8);
    }

    /// Writes an [`i64`] in [var-int](https://wiki.vg/Protocol#VarInt_and_VarLong) format to the underlying buffer.
    fn write_var_i64(&mut self, mut value: i64) {
        while value & -128i64 != 0 {
            self.write_u8(((value & 127i64) as u8) | 128u8);
            value = ((value as u64) >> 7) as i64;
        }
        self.write_u8(value as u8);
    }
}

impl<T: BufMut> PacketBufferWrite for T {
    fn write_i8(&mut self, value: i8) {
        self.put_i8(value);
    }

    fn write_u8(&mut self, value: u8) {
        self.put_u8(value);
    }

    fn write_i16(&mut self, value: i16) {
        self.put_i16(value);
    }

    fn write_u16(&mut self, value: u16) {
        self.put_u16(value);
    }

    fn write_i32(&mut self, value: i32) {
        self.put_i32(value);
    }

    fn write_i64(&mut self, value: i64) {
        self.put_i64(value);
    }

    fn write_f32(&mut self, value: f32) {
        self.put_f32(value);
    }

    fn write_f64(&mut self, value: f64) {
        self.put_f64(value);
    }

    fn write_uuid(&mut self, uuid: &Uuid) {
        self.put_u128(uuid.as_u128());
    }

    fn write_u8_array(&mut self, array: &[u8]) {
        self.put_slice(array);
    }
}
