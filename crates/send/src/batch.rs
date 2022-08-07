use bytes::Bytes;
use falcon_core::network::connection::ConnectionLogic;

pub fn send_batch<B, C, L>(batch: Vec<B>, mut convert: C, connection: &mut L)
where
    C: FnMut(B) -> Option<Bytes>,
    L: ConnectionLogic,
{
    let mut packets = Vec::with_capacity(batch.len());
    for item in batch {
        if let Some(data) = convert(item) {
            packets.push(data);
        }
    }
    // trace!("Writing batch");
    for packet in packets {
        connection.send(&packet);
    }
    // trace!("Batch written, flushing now");
}
