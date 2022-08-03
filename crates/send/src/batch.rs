use bytes::Bytes;
use falcon_core::network::connection::{ConnectionWrapper, ConnectionLogic, ConnectionDriver};


pub fn send_batch<B, C, D, L>(batch: Vec<B>, mut convert: C, connection: &ConnectionWrapper<D, L>)
where
    C: FnMut(B) -> Option<Bytes>,
    D: ConnectionDriver,
    L: ConnectionLogic<D>,
{
    let mut packets = Vec::with_capacity(batch.len());
    for item in batch {
        if let Some(data) = convert(item) {
            packets.push(data);
        }
    }
    connection.execute_sync(move |connection| {
        // trace!("Writing batch");
        for packet in packets {
            connection.driver_mut().send(&packet);
        }
        // trace!("Batch written, flushing now");
    })
}
