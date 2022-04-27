use bytes::Bytes;
use falcon_core::network::connection::ConnectionWrapper;


pub fn send_batch<B, C>(batch: Vec<B>, mut convert: C, connection: &ConnectionWrapper)
where
    C: FnMut(B) -> Option<Bytes>,
{
    let mut packets = Vec::with_capacity(batch.len());
    for item in batch {
        if let Some(data) = convert(item) {
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
