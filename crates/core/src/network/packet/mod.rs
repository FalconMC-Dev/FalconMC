use std::borrow::Cow;
use std::fmt::Debug;
use bytes::Bytes;
pub use falcon_core_derive::{PacketDecode, PacketEncode};
use uuid::Uuid;
use falcon_core::network::connection::{ConnectionDriver, ConnectionLogic};

use crate::error::Result;
use crate::network::buffer::{PacketBufferRead, PacketBufferWrite};

mod packet_macros;

/// Serializes a type to a network buffer.
pub trait PacketEncode {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite);
}

/// Deserializes a type from a network buffer.
pub trait PacketDecode: Sized {
    fn from_buf(buf: &mut dyn PacketBufferRead) -> Result<Self>;
}

/// This trait defines the packet logic when a packet gets received.
pub trait PacketHandler<D: ConnectionDriver, L: ConnectionLogic<D>> {
    /// Executes packet logic.
    fn handle_packet(self, connection: &mut L) -> TaskScheduleResult;

    /// Human-readable identifier of the packet type
    fn get_name(&self) -> &'static str;
}

pub type TaskScheduleResult = std::result::Result<(), TaskScheduleError>;

#[derive(thiserror::Error, Clone, Debug)]
pub enum TaskScheduleError {
    #[error("Could not send task to server thread")]
    ThreadSendError,
    #[error("Error while parsing packet")]
    Message(Cow<'static, str>),
}

impl_packet_primitive_self!(u8, write_u8, read_u8);
impl_packet_primitive_self!(i8, write_i8, read_i8);
impl_packet_primitive_self!(u16, write_u16, read_u16);
impl_packet_primitive_self!(i16, write_i16, read_i16);
impl_packet_primitive_self!(i32, write_i32, read_i32);
impl_packet_primitive_self!(i64, write_i64, read_i64);
impl_packet_primitive_self!(f32, write_f32, read_f32);
impl_packet_primitive_self!(f64, write_f64, read_f64);
impl_packet_primitive_self!(bool, write_bool, read_bool);
impl_packet_decode_primitive_self!(::uuid::Uuid, read_uuid);

impl PacketEncode for Uuid {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        buf.write_uuid(self)
    }
}

impl PacketEncode for Bytes {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        buf.write_u8_array(self.as_ref())
    }
}

/// Vec<u8> can be more efficient than this
impl<T: PacketEncode> PacketEncode for Vec<T> {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        for element in self {
            element.to_buf(buf)
        }
    }
}

impl<T: PacketEncode> PacketEncode for Option<T> {
    fn to_buf(&self, buf: &mut dyn PacketBufferWrite) {
        if let Some(value) = self {
            value.to_buf(buf);
        }
    }
}
