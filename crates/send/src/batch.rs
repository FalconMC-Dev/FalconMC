use bytes::Bytes;
use falcon_core::network::connection::ConnectionWrapper;


pub fn send_batch<B, S, C, F>(batch: Vec<B>, mut convert: C, packet_fn: F, protocol_id: i32, connection: &ConnectionWrapper)
where
    C: FnMut(B) -> S,
    F: Fn(S, i32) -> Option<Bytes>,
{
    let mut packets = Vec::with_capacity(batch.len());
    for item in batch {
        if let Some(data) = packet_fn(convert(item), protocol_id) {
            packets.push(data);
        }
    }
    connection.execute(move |connection| {
        // trace!("Writing batch");
        for packet in packets {
            connection.send_data(&packet);
        }
        // trace!("Batch written, flushing now");
    })
}