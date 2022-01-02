use bytes::Buf;
use uuid::Uuid;

use crate::errors::*;
use error_chain::{bail, ensure};

pub trait PacketBufferRead {
    /// Reads an [`i8`] from the underlying buffer.
    fn read_i8(&mut self) -> Result<i8>;

    /// Reads an [`u8`] from the underlying buffer.
    fn read_u8(&mut self) -> Result<u8>;

    /// Reads an [`i16`] from the underlying buffer.
    fn read_i16(&mut self) -> Result<i16>;

    /// Reads an [`u16`] from the underlying buffer.
    fn read_u16(&mut self) -> Result<u16>;

    /// Reads an [`i32`] from the underlying buffer.
    fn read_i32(&mut self) -> Result<i32>;

    /// Reads an [`i32`] from the underlying buffer.
    fn read_i64(&mut self) -> Result<i64>;

    /// Reads an [`f32`] from the underlying buffer.
    fn read_f32(&mut self) -> Result<f32>;

    /// Reads an [`f64`] from the underlying buffer.
    fn read_f64(&mut self) -> Result<f64>;

    /// Reads a [`Uuid`] from the underlying buffer.
    fn read_uuid(&mut self) -> Result<Uuid>;

    /// Reads a byte array with the specified length from the underlying buffer.
    fn read_u8_array(&mut self, length: usize) -> Result<Vec<u8>>;

    /// Reads a [`bool`] from the underlying buffer.
    fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? == 1)
    }

    /// Reads a [`String`] with specified max length from the underlying buffer.
    fn read_string(&mut self, max_length: i32) -> Result<String> {
        let len = self.read_var_i32()?;
        if len == 0 {
            bail!(ErrorKind::StringSizeZero);
        }
        if len > max_length * 4 + 3 {
            bail!(ErrorKind::StringTooLong);
        }
        let result = String::from_utf8(self.read_u8_array(len as usize)?)
            .chain_err(|| ErrorKind::BadString)?;
        if result.chars().count() > max_length as usize {
            bail!(ErrorKind::StringTooLong);
        }
        Ok(result)
    }

    // TODO: add chat struct
    //fn read_chat(&mut self) -> Result<Chat>;

    // TODO: add identifier struct
    //fn read_identifier(&mut self) -> Result<Identifier>;

    /// Reads an [`i32`] in [var-int](https://wiki.vg/Protocol#VarInt_and_VarLong) format from the underlying buffer.
    fn read_var_i32(&mut self) -> Result<i32> {
        let mut result: i32 = 0;
        for i in 0..=6 {
            if i > 5 {
                bail!(ErrorKind::VarI32TooLong);
            }
            let byte = self.read_u8()?;
            result |= ((byte & 0b0111_1111) as i32) << (i * 7);
            if byte & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    /// Reads an [`i64`] in [var-int](https://wiki.vg/Protocol#VarInt_and_VarLong) format from the underlying buffer.
    fn read_var_i64(&mut self) -> Result<i64> {
        let mut result: i64 = 0;
        for i in 0..=11 {
            if i > 10 {
                bail!(ErrorKind::VarI64TooLong);
            }
            let byte = self.read_u8()?;
            result |= ((byte & 0b0111_1111) as i64) << (i * 7);
            if byte & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    // TODO: add block_pos struct
    //fn read_block_pos(&mut self) -> Result<BlockPos>;

    fn remaining_bytes(&self) -> usize;
}

impl<T: Buf> PacketBufferRead for T {
    fn read_i8(&mut self) -> Result<i8> {
        self.ensure_bytes_available(1)?;
        Ok(self.get_i8())
    }

    fn read_u8(&mut self) -> Result<u8> {
        self.ensure_bytes_available(1)?;
        Ok(self.get_u8())
    }

    fn read_i16(&mut self) -> Result<i16> {
        self.ensure_bytes_available(2)?;
        Ok(self.get_i16())
    }

    fn read_u16(&mut self) -> Result<u16> {
        self.ensure_bytes_available(2)?;
        Ok(self.get_u16())
    }

    fn read_i32(&mut self) -> Result<i32> {
        self.ensure_bytes_available(4)?;
        Ok(self.get_i32())
    }

    fn read_i64(&mut self) -> Result<i64> {
        self.ensure_bytes_available(8)?;
        Ok(self.get_i64())
    }

    fn read_f32(&mut self) -> Result<f32> {
        self.ensure_bytes_available(4)?;
        Ok(self.get_f32())
    }

    fn read_f64(&mut self) -> Result<f64> {
        self.ensure_bytes_available(8)?;
        Ok(self.get_f64())
    }

    fn read_uuid(&mut self) -> Result<Uuid> {
        self.ensure_bytes_available(16)?;
        Ok(Uuid::from_u128(self.get_u128()))
    }

    fn read_u8_array(&mut self, length: usize) -> Result<Vec<u8>> {
        if length == 0 {
            return Ok(vec![]);
        }
        self.ensure_bytes_available(length)?;
        let mut result = Vec::with_capacity(length);
        result.extend(self.copy_to_bytes(length));
        Ok(result)
    }

    fn remaining_bytes(&self) -> usize {
        self.remaining()
    }
}

pub trait ByteLimitCheck {
    /// Returns [`ErrorKind::NoMoreBytes`] when the amount of bytes requested is not available from the underlying buffer.
    fn ensure_bytes_available(&self, amount: usize) -> Result<()>;
}

impl<T: Buf> ByteLimitCheck for T {
    fn ensure_bytes_available(&self, amount: usize) -> Result<()> {
        ensure!(self.remaining() >= amount, ErrorKind::NoMoreBytes);
        Ok(())
    }
}
