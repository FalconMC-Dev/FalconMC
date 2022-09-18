use bytes::Bytes;
use falcon_core::network::connection::ConnectionLogic;
use tracing::{instrument, trace};

#[instrument(level = "trace", target = "metrics::send", skip_all, fields(count = batch.len()))]
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
    trace!(target: "metrics::send::batch", "Writing batch");
    for packet in packets {
        connection.send(&packet);
    }
    trace!(target: "metrics::send::batch", "Flushing batch");
}
