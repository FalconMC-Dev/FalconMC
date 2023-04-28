/// Helper trait for implementing the [`PacketRead`] trait on a type.
///
/// See [`packet`](crate::packet) for full expansions of each packet type.
///
/// # Example
/// ```no_run
/// # use falcon_packet::packet;
/// # use falcon_packet::PacketRead;
/// packet! {
///     pub struct StructExample {
///         num: i32,
///         let plus_five: i32 = num + 5,
///     }
/// }
///
/// # use bytes::Buf;
/// # use falcon_packet::ReadError;
/// # use falcon_packet::read;
/// impl PacketRead for StructExample {
///     fn read<B>(buffer: &mut B) -> Result<Self, ReadError>
///     where
///         B: Buf + ?Sized,
///         Self: Sized,
///     {
///         read! {
///             var32 => num: i32,
///             self => plus_five: i32,
///         }
///         Ok(StructExample { num, plus_five, })
///     }
/// }
/// ```
///
/// [`PacketRead`]: (crate::PacketRead)
pub use falcon_packet_derive::read;
/// Helper trait for implementing the [`PacketSize`] trait on a type.
///
/// See [`packet`](crate::packet) for full expansions of each packet type.
///
/// # Example
/// ```
/// # use falcon_packet::packet;
/// # use falcon_packet::PacketSize;
/// packet! {
///     pub struct StructExample {
///         num: i32,
///     }
/// }
///
/// # use falcon_packet::size;
/// impl PacketSize for StructExample {
///     fn size(&self) -> usize {
///         size!(
///             var32 => i32 = self.num,
///             self => i32 = self.num + 5,
///         ) + 10 // random 10 for example purposes
///     }
/// }
///
/// let example = StructExample::new(10);
/// assert_eq!(15, example.size());
/// ```
///
/// [`PacketSize`]: (crate::PacketSize)
pub use falcon_packet_derive::size;
/// Helper trait for implementing the [`PacketWrite`] trait on a type.
///
/// See [`packet`](crate::packet) for full expansions of each packet type.
///
/// # Example
/// ```no_run
/// # use falcon_packet::packet;
/// # use falcon_packet::PacketWrite;
/// packet! {
///     pub struct StructExample {
///         num: i32,
///     }
/// }
///
/// # use bytes::BufMut;
/// # use falcon_packet::WriteError;
/// # use falcon_packet::write;
/// impl PacketWrite for StructExample {
///     fn write<B>(&self, buffer: &mut B) -> Result<(), WriteError>
///     where
///         B: BufMut
///     {
///         write! {
///             var32 => i32 = self.num,
///             self => i32 = self.num + 5,
///         }
///         Ok(())
///     }
/// }
/// # use falcon_packet::PacketSize;
/// # use falcon_packet::size;
/// # impl PacketSize for StructExample {
/// #     fn size(&self) -> usize {
/// #         size!(
/// #             var32 => i32 = self.num,
/// #             self => i32 = self.num + 5,
/// #         )
/// #     }
/// # }
/// ```
///
/// [`PacketWrite`]: (crate::PacketWrite)
pub use falcon_packet_derive::write;
