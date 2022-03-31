use std::thread;

use ignore_result::Ignore;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use falcon_core::ShutdownHandle;

use anyhow::{Result, Context};

pub struct ConsoleListener {
    shutdown_handle: broadcast::Sender<()>,
    console_sender: UnboundedSender<String>,
}

impl ConsoleListener {
    pub fn start_console(shutdown_handle: ShutdownHandle) -> Result<UnboundedReceiver<String>> {
        info!("Starting console thread!");
        let (console_tx, console_rx) = unbounded_channel();

        let console = ConsoleListener {
            shutdown_handle: shutdown_handle.into_signal_sender(),
            console_sender: console_tx,
        };
        thread::Builder::new()
            .name(String::from("Console listener"))
            .spawn(|| console.start_reading())
            .with_context(|| "Couldn't start console listener!")?;

        Ok(console_rx)
    }

    #[tracing::instrument(name = "console", skip_all)]
    fn start_reading(self) {
        loop {
            let mut buffer = String::new();
            let stdin = std::io::stdin();
            if let Err(ref e) = stdin
                .read_line(&mut buffer)
                .with_context(|| "Could not read from stdin!")
            {
                print_error!(e);
                self.shutdown_handle.send(()).ignore();
                break;
            } else {
                trace!(input = %buffer, "Sending console input!");
                if self.console_sender.send(buffer).is_err() {
                    break;
                };
            }
        }
    }
}
