use crate::errors::*;
use crossbeam::channel::Receiver;
use falcon_core::server::{McTask, MinecraftServer};
use std::thread;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::time::MissedTickBehavior;

use crate::network::listener::NetworkListener;
use falcon_core::ShutdownHandle;

pub struct MainServer {
    shutdown_handle: ShutdownHandle,
    server_rx: Receiver<Box<McTask>>,
}

impl MainServer {
    pub fn start_server(shutdown_handle: ShutdownHandle) -> Result<()> {
        info!("Starting server thread...");

        let (server_tx, server_rx) = crossbeam::channel::unbounded();
        let server = MainServer {
            shutdown_handle,
            server_rx,
        };

        tokio::spawn(NetworkListener::start_network_listening(
            server.shutdown_handle.clone(),
            server_tx,
        ));

        thread::Builder::new()
            .name(String::from("Main Server Thread"))
            .spawn(|| server.start_server_logic())
            .chain_err(|| "Couldn't start server logic!")?;

        Ok(())
    }

    fn start_server_logic(mut self) {
        debug!("Starting server logic!");
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(50));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                self.tick();

                tokio::select! {
                    _ = interval.tick() => {}
                    _ = self.shutdown_handle.wait_for_shutdown() => {
                        debug!("Stopping server logic!");
                        break;
                    }
                }
            }
        });
    }

    fn tick(&mut self) {
        while let Ok(task) = self.server_rx.try_recv() {
            task(self);
        }
    }
}

impl MinecraftServer for MainServer {}
