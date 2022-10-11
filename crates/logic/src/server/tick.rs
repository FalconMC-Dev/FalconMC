use std::str::FromStr;
use std::time::Duration;

use tokio::runtime::Builder;
use tokio::time::MissedTickBehavior;
use tracing::{debug, debug_span, error, info};

use super::ServerTask;
use crate::server::command::Command;
use crate::FalconServer;

impl FalconServer {
    #[tracing::instrument(name = "server", skip(self))]
    pub fn start(&mut self) {
        debug!("Starting server logic!");
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_millis(50));
            tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let mut keep_alive_interval = tokio::time::interval(Duration::from_secs(12));
            keep_alive_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            while !self.should_stop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        self.tick().await;
                    }
                    _ = keep_alive_interval.tick() => {
                        self.keep_alive();
                    }
                    _ = self.shutdown_handle().wait_for_shutdown() => {
                        break;
                    }
                }
            }
            debug!("Stopping server logic!");
        });
    }

    /// Game loop method
    #[tracing::instrument(skip(self), fields(player_count = self.online_count()))]
    async fn tick(&mut self) {
        while let Ok(task) = self.receiver.try_recv() {
            let span = debug_span!("server_task");
            let _enter = span.enter();
            if let Err(error) = match task {
                ServerTask::Sync(task) => task.run(self),
                ServerTask::Async(task) => task.run(self).await,
            } {
                error!(%error);
            }
        }
        while let Ok(command) = self.console_rx.try_recv() {
            info!(cmd = %command.trim(), "Console command execution");
            // TODO: more commands
            let command_parsed: Command = match Command::from_str(command.as_str()) {
                Ok(val) => val,
                Err(e) => {
                    error!(error = %e, "Error when parsing command");
                    continue;
                },
            };
            match command_parsed {
                Command::Stop => {
                    info!("Shutting down server! (Stop command executed)");
                    self.should_stop = true;
                    self.shutdown_handle().send_shutdown();
                },
                Command::Kick(username) => {
                    todo!()
                },
            }
        }
    }

    #[tracing::instrument(skip(self), fields(player_count = self.players.len()))]
    fn keep_alive(&mut self) { self.players.values().for_each(|player| player.send_keep_alive()); }
}
