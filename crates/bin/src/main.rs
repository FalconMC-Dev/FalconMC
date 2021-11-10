#[macro_use]
extern crate log;

mod errors;
mod server;

use errors::*;

use falcon_core::ShutdownHandle;

#[tokio::main]
async fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
    info!("Launching Falcon Server!");

    let (mut shutdown_handle, mut finished_rx) = ShutdownHandle::new();

    std::mem::drop(shutdown_handle);
    let _ = finished_rx.recv().await;
    debug!("End of application!");
}