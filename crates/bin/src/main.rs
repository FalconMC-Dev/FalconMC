#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

mod errors;
mod network;
mod server;

use crate::server::MainServer;
use falcon_core::ShutdownHandle;

#[tokio::main]
async fn main() {
    log4rs::init_file(falcon_core::LOG_CONFIG, Default::default()).unwrap();
    info!("Launching Falcon Server!");

    let (mut shutdown_handle, mut finished_rx) = ShutdownHandle::new();
    if let Err(ref e) = MainServer::start_server(shutdown_handle.clone()) {
        print_error!(e);
        shutdown_handle.send_shutdown();
    } else {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down server! (Ctrl-C exited)");
                shutdown_handle.send_shutdown();
            }
            _ = shutdown_handle.wait_for_shutdown() => {}
        }
    }

    std::mem::drop(shutdown_handle);
    let _ = finished_rx.recv().await;
    info!("Falcon Server has shut down!");
}
