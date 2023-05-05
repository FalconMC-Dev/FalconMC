/// Packet reading delegation macro
///
/// To speed up reading a packet from the network based
/// on a protocol version and a packet id, this macro can be used
/// to efficiently generate a parsing function.
///
/// The syntax this macro accepts is a list of mappings between protocol
/// versions and packet ids associated with *packet types* (i.e. types
/// that implements [`PacketRead`]).
///
/// Here's an example:
/// ```no_run
/// # use falcon_protocol::packets;
/// # use falcon_packet::packet;
/// # packet! {
/// #     pub packet struct PacketOne {
/// #         self => x: i32,
/// #     }
/// # }
/// # packet! {
/// #     pub packet struct PacketTwo {
/// #         self => x: i32,
/// #     }
/// # }
/// packets! {
///     PacketOne: 47, 753 = 0x02,
///     PacketTwo: {
///         47, 48 = 0x03;
///         50, 51 = 0x02;
///     }
/// }
/// ```
///
/// If there are multiple `... = ...` mappings, braces are required.
///
/// The output of this macro will be a function called
/// `read_packet` that takes in a [`&mut Buf`](bytes::Buf), a
/// packet_id `i32` and a protocol version `i32`. The return
/// value of that function is a
/// [`Result<Option<Box<dyn Packet>>, ReadError>`](falcon_protocol::Packet).
///
/// ```no_run
/// # use falcon_protocol::Packet;
/// # use falcon_packet::ReadError;
/// pub fn read_packet<B>(buffer: &mut B, packet_id: i32, protocol_version: i32) -> Result<Option<Box<dyn Packet>>, ReadError>
/// where
///     B: ::bytes::Buf,
/// {
///     // impl details omitted
///     # Ok(None)
/// }
/// ```
///
/// [`PacketRead`]: (falcon_packet::PacketRead)
pub use falcon_protocol_derive::packets;
