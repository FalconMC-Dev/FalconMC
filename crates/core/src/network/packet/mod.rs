use crate::errors::*;

mod packet_macros;

use crate::network::buffer::{PacketBufferRead, PacketBufferWrite};
use crate::network::PacketHandlerState;
use crate::server::McTask;
use crossbeam::channel::Sender;
pub use falcon_core_derive::{PacketDecode, PacketEncode};

/// Defines the ID of a packet.\
/// All outgoing packets should implement this trait.
pub trait PacketId {
    fn get_packet_id(&self) -> i32;
}

/// Serializes a type to a network buffer.
pub trait PacketEncode {
    fn to_buf(self, buf: &mut dyn PacketBufferWrite);
}

/// Deserializes a type from a network buffer.
pub trait PacketDecode: Sized {
    fn from_buf(buf: &mut dyn PacketBufferRead) -> Result<Self>;
}

/// This trait defines the packet logic when a packet gets received.
pub trait PacketHandler {
    /// Executes packet logic.
    fn handle_packet(&mut self, state: &mut PacketHandlerState, task_tx: &mut Sender<Box<McTask>>);

    /// Human-readable identifier of the packet type
    fn get_name(&self) -> &'static str;
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
impl_packet_primitive_self!(uuid::Uuid, write_uuid, read_uuid);
