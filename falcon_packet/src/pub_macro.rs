/// Builder macro for packet structs.
///
/// This macro is designed to aid the creation of
/// packet structs as specified by the [Minecraft protocol](https://wiki.vg/).
///
/// The output of this macro consists of a struct definition with a
/// provided `new` function. A [`PacketSize`], [`PacketWrite`] and
/// [`PacketRead`] implementation also gets generated.
///
/// The accepted syntax of the macro is best illustrated by examples:
///
/// ## Defining a struct field
///
/// There are two ways to define a struct field: the initializer syntax and the
/// constructor syntax. The first syntax can be used to allow custom
/// initialization for that field based on the local scope (other arguments in
/// the `new()` function etc.). The constructor syntax makes the field take its
/// value from a parameter in the provided `new()` function.
/// ```no_run
/// # use falcon_packet::packet;
/// // first field uses constructor syntax, second field uses
/// // initializer syntax referencing the first field
/// packet! {
///     pub packet struct PacketExample {
///         self => num: i32,
///         self => let plus_five: i32 = num + 5,
///     }
/// }
///
/// // `PacketExample::new()` expects one `i32`
/// let packet = PacketExample::new(10);
///
/// // The order doesn't matter for initialization here because
/// // the `num` variable is taken from the parameter list of `new()`.
/// packet! {
///     pub packet struct PacketExampleSwapped {
///         self => let plus_five: i32 = num + 5,
///         self => num: i32,
///     }
/// }
///
/// // `PacketExample::new()` expects one `i32`
/// let packet = PacketExampleSwapped::new(10);
/// ```
/// - The visilibity before the `struct` keyword works like with normal structs.
/// - Instead of defining every field for the Rust struct, the macro expects a
///   definition of every field for the protocol. This is why it first expects a
///   notation denoting how to write the field to the protocol before defining
///   the field of the rust struct. See below for all supported notations.
/// - **All fields are `pub`**.
/// - The order the protocol fields are defined in determines the order in which
///   the fields are written to the network.
///
/// ## Extra parameters
///
/// Extra parameters for the `new()` function can be specified like this:
/// ```no_run
/// # use falcon_packet::packet;
/// packet! {
///     pub packet struct PacketExample => num: i32, _unused: u8 {
///         self => let plus_five: i32 = num + 5,
///         self => constructor_field: u8
///     }
/// }
///
/// // extra parameters come first
/// let packet = PacketExample::new(-10, 8, 0); // (num, _unused, constructor_field)
/// ```
///
/// # Field notation
/// The macro is designed to bridge between protocol fields and rust struct
/// fields. The keyword before `=>` denotes how to write the next protocol
/// field, there are the following possibilities:
///
/// - **`self`**:
///
///     ```no_run
///     # type Type = u8;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     self => field: Type
///     #   }
///     # }
///     ```
///     The field is read and written using the [`PacketRead`] and
///     [`PacketWrite`] implementations of the type from the rust field.
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "self => field: Type",
///     // Read expands as:
///     # stringify!(
///     let field = ::falcon_packet::PacketRead::read(buffer)?;
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::PacketWrite::write(&self.field, buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::PacketSize::size(&self.field)
///     # ));
///     ```
///     <br>
/// - **`self as Type`**:
///
///     ```no_run
///     # type Type = u8;
///     # type OtherType = u16;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     self as OtherType => field: Type
///     #   }
///     # }
///     ```
///     The field is read and written using the [`PacketRead`] and
///     [`PacketWrite`] implementations of the other type specified using
///     an `as` conversion.
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "self as OtherType => field: Type",
///     // Read expands as:
///     # stringify!(
///     let field = <OtherType as ::falcon_packet::PacketRead>::read(buffer)? as Type;
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::PacketWrite::write(&(self.field as OtherType), buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::PacketSize::size(&(self.field as OtherType))
///     # ));
///     ```
///     <br>
/// - **`var32/var64`**:
///
///     ```no_run
///     # type Type = i32;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     var32 => field: Type
///     #   }
///     # }
///     ```
///     The field is transformed into a [`VarI32`]
///     or [`VarI64`] first. For now this field
///     must be copy. This shouldn't be a problem since it's mostly integers
///     that should be handled this way, if it is a problem, open an issue.
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "var32 => field: Type",
///     // Read expands as:
///     # stringify!(
///     let field = <::falcon_packet::primitives::VarI32 as ::falcon_packet::PacketRead>::read(buffer)?.into();
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::PacketWrite::write(&<Type as Into<::falcon_packet::primitives::VarI32>>::into(self.field), buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::PacketSize::size(&<Type as Into<::falcon_packet::primitives::VarI32>>::into(self.field))
///     # ));
///     ```
///     <br>
/// - **`str`** or **`str(max_len)`**:
///
///     ```no_run
///     # type Type = falcon_packet::primitives::PacketString;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     str(10) => field: Type
///     #   }
///     # }
///     ```
///     Writes a type using the string representation of the protocol.
///     There is a maximum length specified for most fields which can be
///     left out in which case it will be a default length of 32767 bytes.
///     Consider using [`PacketString`] as a convenience wrapper.
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "str(10) => field: Type",
///     // Read expands as:
///     # stringify!(
///     let field = ::falcon_packet::PacketReadSeed::read(10usize, buffer)?;
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::PacketWriteSeed::write(10usize, <Type as AsRef<str>>::as_ref(&self.field), buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::PacketSize::size(<Type as AsRef<str>>::as_ref(&self.field))
///     # ));
///     ```
///     <br>
/// - **`bytes`**:
///
///     ```no_run
///     # type Type = falcon_packet::primitives::PacketBytes;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     #       var32 => len_field: usize,
///     bytes => { field: Type, len_field = self.field.len() }
///     #   }
///     # }
///     ```
///     This is perhaps one of the trickiest primitive types
///     to serialize for the protocol. This type isn't defined
///     as always having its length as a prefix unlike strings. For
///     this reason the field its length should be read from needs
///     to be specified manually together with the expression
///     that determines the type's length upon writing.
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "bytes => { field: Type, len_field = self.field.len() }",
///     // Read expands as:
///     # stringify!(
///     let field = ::falcon_packet::PacketReadSeed::read(len_field as usize, buffer)?;
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::PacketWrite::write(<Type as AsRef<[u8]>>::as_ref(&self.field), buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::PacketSize::size(&self.field)
///     # ));
///     ```
///     <br>
/// - **`rest`**:
///
///     ```no_run
///     # type Type = falcon_packet::primitives::PacketBytes;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     rest => field: Type
///     #   }
///     # }
///     ```
///     This should be **the last field** of a packet. It reads
///     all bytes that are left in the packet.
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "rest => field: Type",
///     // Read expands as:
///     # stringify!(
///     let field = ::falcon_packet::PacketReadSeed::read((), buffer)?;
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::PacketWrite::write(<Type as AsRef<[u8]>>::as_ref(&self.field), buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::PacketSize::size(&self.field)
///     # ));
///     ```
///     <br>
/// - **`array/bytearray`**:
///
///     ```no_run
///     # type Type = i32;
///     # const N: usize = 3;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     array => field: [Type; N]
///     #   }
///     # }
///     ```
///     Reads and writes an array of `Type` from the packet. If the field
///     is an array of bytes, always use `bytearray` for better performance.
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "array => field: [Type; N]",
///     // Read expands as:
///     # stringify!(
///     let field = ::falcon_packet::primitives::array_read(buffer)?;
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::PacketWrite::write(&self.field, buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::PacketSize::size(&self.field)
///     # ));
///     ```
///     <br>
/// - **`nbt`**:
///
///     ```no_run
///     # type Type = String;
///     # falcon_packet::packet! {
///     #   pub packet struct PacketRead {
///     nbt => field: Type
///     #   }
///     # }
///     ```
///     Reads and writes a `Type` from the packet using [`fastnbt`].
///     ```
///     # falcon_packet_core::doctest_impls(
///     # "nbt => field: Type",
///     // Read expands as:
///     # stringify!(
///     let field = ::falcon_packet::primitives::nbt_read(buffer)?;
///     # ),
///
///     // Write expands as:
///     # stringify!(
///     ::falcon_packet::primitives::nbt_write(&self.field, buffer)?;
///     # ),
///
///     // Size expands as:
///     # stringify!(
///     ::falcon_packet::primitives::nbt_size(&self.field)
///     # ));
///     ```
///     <br>
///
/// [`PacketRead`]: super::PacketRead
/// [`PacketWrite`]: super::PacketWrite
/// [`PacketSize`]: super::PacketSize
/// [`VarI32`]: crate::primitives::VarI32
/// [`VarI64`]: crate::primitives::VarI64
/// [`PacketString`]: crate::primitives::PacketString
pub use falcon_packet_derive::packet;
