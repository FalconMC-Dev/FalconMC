//! Extra types that are are not considered primitives. Also contains utility
//! types used in the [derive macros](falcon_packet_core#derives).
//!
//! **Data types**:
//! - [`StrUuid`]: String representation of [`Uuid`](::uuid::Uuid).
//!
//! **Traits**:
//! - [`PacketPrepare`]: Used to turn [`BufMut`](bytes::BufMut) into a packet
//!   sink.
//!
//! **Utility types**:
//! - [`Counter`]: Counts bytes that are written to it.
//! - [`Reader`]: Wrapper around [`Buf`](bytes::Buf).
//! - [`Writer`]: Wrapper around [`BufMut`](bytes::BufMut).

mod counter;
mod packet;
mod reader;
mod uuid;
mod writer;

pub use self::counter::Counter;
pub use self::packet::PacketPrepare;
pub use self::reader::Reader;
pub use self::uuid::StrUuid;
pub use self::writer::Writer;
