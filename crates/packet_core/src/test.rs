use bytes::{Buf, BufMut};
use falcon_packet_core_derive::{PacketRead, PacketSize, PacketWrite};

use crate::ReadError;

#[derive(PacketSize, PacketWrite, PacketRead)]
pub struct TestPacket {
    #[falcon(array)]
    array: [u8; 2],
    // #[falcon(var32)]
    id: i32,
    #[falcon(var32)]
    length: usize,
    #[falcon(var32)]
    length2: usize,
    #[falcon(bytes)]
    test: Vec<u8>,
    #[falcon(vec = "length2")]
    test2: Vec<u8>,
    #[falcon(convert = "String", string = 40)]
    name: TestStrWrapper,
    #[falcon(string = 20)]
    ref_test: TestStrWrapper,
    #[falcon(link = "length, id with link_fn")]
    link_test: u32,
    // #[falcon(nbt)]
    // nbt_test: TestStrWrapper,
}

fn link_fn_value(field: &u32, _id: &i32) -> usize { field.leading_zeros() as usize }

fn link_fn_size(field: &u32) -> usize { ::falcon_packet_core::PacketSize::size(field) }

fn link_fn_write<B: BufMut + ?Sized>(field: &u32, buffer: &mut B) -> Result<(), crate::WriteError> { crate::PacketWrite::write(field, buffer) }

fn link_fn_read<B: Buf + ?Sized>(buffer: &mut B, length: &usize, _id: &i32) -> Result<u32, ReadError> {
    println!("value: {}", length);
    crate::PacketRead::read(buffer)
}

#[derive(Clone)]
struct TestStrWrapper {
    content: String,
}

impl From<TestStrWrapper> for String {
    fn from(data: TestStrWrapper) -> Self { data.content }
}

impl From<String> for TestStrWrapper {
    fn from(data: String) -> Self { Self { content: data } }
}

impl AsRef<str> for TestStrWrapper {
    fn as_ref(&self) -> &str { &self.content }
}
