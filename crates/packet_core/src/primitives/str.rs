use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, BufMut};

use crate::error::{ReadError, WriteError};
use crate::{
    PacketRead, PacketReadSeed, PacketSize, PacketVec, PacketWrite, PacketWriteSeed, VarI32,
};

pub struct AsRefStr<T>(pub T);

impl<T> Deref for AsRefStr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AsRefStr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsRef<str>> AsRef<str> for AsRefStr<T> {
    fn as_ref(&self) -> &str {
        self.deref().as_ref()
    }
}

impl<T: AsRef<str>> PacketSize for AsRefStr<T> {
    fn size(&self) -> usize {
        VarI32::from(self.as_ref().len()).size() + self.as_ref().len()
    }
}

pub struct PacketString<T> {
    size: usize,
    _marker: PhantomData<T>,
}

impl<T> PacketString<T> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            _marker: PhantomData,
        }
    }
}

impl<T: AsRef<str>> PacketWriteSeed for PacketString<T> {
    type Value = T;

    fn write<B>(self, value: Self::Value, buffer: &mut B) -> Result<(), WriteError>
    where
        B: BufMut + ?Sized,
    {
        let count = value.as_ref().chars().count();
        if count > self.size {
            Err(WriteError::StringTooLong(self.size, count))
        } else {
            VarI32::from(value.as_ref().len()).write(buffer)?;
            value.as_ref().as_bytes().write(buffer)
        }
    }
}

impl<T: From<String>> PacketReadSeed for PacketString<T> {
    type Value = T;

    fn read<B>(self, buffer: &mut B) -> Result<Self::Value, ReadError>
    where
        B: Buf + ?Sized,
    {
        let len = VarI32::read(buffer)?.as_usize();
        if len > self.size * 4 {
            return Err(ReadError::StringTooLong(self.size * 4, len));
        }
        let buf = PacketVec::new(len).read(buffer)?;
        let str = String::from_utf8(buf)?;
        let count = str.chars().count();
        if count > self.size {
            Err(ReadError::StringTooLong(self.size, count))
        } else {
            Ok(str.into())
        }
    }
}
