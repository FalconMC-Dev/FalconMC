#[macro_use]
extern crate log;

mod errors;
mod server;
mod network;

use errors::*;

use falcon_core::ShutdownHandle;
use crate::server::MainServer;

#[tokio::main]
async fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
    info!("Launching Falcon Server!");

    let (mut shutdown_handle, mut finished_rx) = ShutdownHandle::new();
    if let Err(ref e) = MainServer::start_server(shutdown_handle.clone()) {
        println!("error: {}", e);
        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }
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