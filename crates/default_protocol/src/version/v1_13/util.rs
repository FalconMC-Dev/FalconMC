use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;

use falcon_core::network::connection::{ConnectionTask, MinecraftConnection};
use falcon_core::network::packet::PacketEncode;
use falcon_core::world::chunks::{SECTION_HEIGHT, SECTION_LENGTH, SECTION_WIDTH};

#[inline]
pub fn easy_send<T: PacketEncode + Send + Sync + 'static>(connection: &mut UnboundedSender<Box<ConnectionTask>>, id: i32, packet: T) -> Result<(), SendError<Box<ConnectionTask>>> {
    connection.send(Box::new(move |conn: &mut dyn MinecraftConnection| {
        let packet_out = packet;
        conn.send_packet(id, &packet_out);
    }))
}

pub fn build_compacted_data_array<E: Iterator<Item=u64>>(bits_per_block: u8, elements: E) -> Vec<u64> {
    let long_count: u32 = (SECTION_WIDTH * SECTION_HEIGHT * SECTION_LENGTH * bits_per_block as u16) as u32 / i64::BITS;
    let mut compacted_data = Vec::with_capacity(long_count as usize);
    let mut current_long = 0u64;
    let mut offset = 0;
    let mut pos = 0;

    for element in elements {
        let bit_shift = pos * bits_per_block + offset;
        if bit_shift < (i64::BITS - bits_per_block as u32) as u8 {
            current_long |= element << bit_shift;
            pos += 1;
        } else {
            offset = bit_shift - (i64::BITS - bits_per_block as u32) as u8;
            current_long |= element << bit_shift;
            compacted_data.push(current_long);
            current_long = 0u64;
            if offset != 0 {
                let diff = bits_per_block - offset;
                current_long |= element >> diff;
            }
            pos = 0;
        }
    }

    compacted_data
}
