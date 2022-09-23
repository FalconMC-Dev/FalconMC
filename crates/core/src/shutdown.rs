use std::fmt::{Debug, Formatter};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info};

/// A wrapper around two [Tokio channels](https://docs.rs/tokio/1.13.0/tokio/sync/index.html) to asynchronously control
/// proper application clean-up upon termination.
///
/// `ShutdownHandle` allows us to easily make sure everything is cleaned up across different threads
/// before terminating the running application.
///
/// This handle has no limitation to only be used for shutdown; on the contrary,
/// this is a multithreaded flag that allows signaling all related flags from any related flag
/// and has one master receiver waiting for all flags to signal their tasks having been completed.
///
/// ## Cloning
/// Every thread/task running should own an instance of this struct. Passing
/// related handles across different threads is done by cloning one such handle.
///
/// As long as at least one handle is not dropped, the program will not terminate until all handles
/// are dropped and thus all tasks have ended.
pub struct ShutdownHandle {
    #[doc(hidden)]
    signal_sender: broadcast::Sender<()>,
    #[doc(hidden)]
    signal_receiver: broadcast::Receiver<()>,
    #[doc(hidden)]
    shutdown_finished: mpsc::Sender<()>,
}

impl ShutdownHandle {
    /// Creates a new `ShutdownHandle` instance and returns a `Receiver` along with it.
    ///
    /// This `Receiver` receives a message when all instances related to this handle are dropped.\
    /// Idiomatically, this `Receiver` should be used in the main loop to wait before exiting.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::mem::drop;
    ///
    /// use falcon_core::ShutdownHandle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (shutdown_handle, finish_rx) = ShutdownHandle::new();
    ///
    ///     // Pass this handle through cloning to other tasks of the program
    ///
    ///     // Drop the handle in this scope before waiting
    ///     drop(shutdown_handle);
    ///     let _ = recv.recv().await;
    ///     // everything is guaranteed to have shut down
    /// }
    /// ```
    pub fn new() -> (ShutdownHandle, mpsc::Receiver<()>) {
        let (shutdown_finished_tx, shutdown_finished_rx) = mpsc::channel(1);
        let (sender, receiver) = broadcast::channel(1);
        (
            ShutdownHandle {
                signal_sender: sender,
                signal_receiver: receiver,
                shutdown_finished: shutdown_finished_tx,
            },
            shutdown_finished_rx,
        )
    }

    /// Signals all clones of this `ShutdownHandle` to terminate their associated tasks,
    /// after which each `ShutdownHandle` should be dropped.
    ///
    /// # Examples
    /// ```ignore
    /// use std::mem::drop;
    ///
    /// use falcon_core::ShutdownHandle;
    ///
    /// async fn do_something() {
    ///     let (shutdown_handle, recv) = ShutdownHandle::new();
    ///
    ///     // Pass this handle through cloning to all other tasks of the program
    ///
    ///     // when we want to terminate
    ///     shutdown_handle.send_shutdown();
    ///
    ///     // now we can wait for everything to have shutdown as explained in `ShutdownHandle::new()`
    /// }
    pub fn send_shutdown(&self) {
        info!("Shutdown requested!");
        let _ = self.signal_sender.send(());
    }

    /// Returns `true` if a shutdown signal has already been received, and `false` otherwise.
    pub fn try_is_shutdown(&mut self) -> bool {
        if let Err(error) = self.signal_receiver.try_recv() {
            if error == broadcast::error::TryRecvError::Empty {
                return false;
            }
        }
        true
    }

    /// Waits asynchronously for this handle to receive a shutdown signal before returning.
    pub async fn wait_for_shutdown(&mut self) -> Result<(), broadcast::error::RecvError> {
        self.signal_receiver.recv().await
    }

    /// Consumes self and returns a channel that can be used to trigger a shutdown.
    ///
    /// This is useful for threads that are dependent on the main thread but cannot terminate on their own.\
    /// This can also be used to pass on a bare trigger for shutdown by cloning self first and then calling this function.
    ///
    /// # Note
    /// This shutdown handler will be dropped and won't signal a process complete anymore.\
    /// There is also no way to receive a shutdown signal either (unless you clone this first).
    pub fn into_signal_sender(self) -> broadcast::Sender<()> {
        self.signal_sender.clone()
    }
}

impl Debug for ShutdownHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShutdownHandle").finish()
    }
}

impl std::clone::Clone for ShutdownHandle {
    fn clone(&self) -> Self {
        ShutdownHandle {
            signal_sender: self.signal_sender.clone(),
            signal_receiver: self.signal_sender.subscribe(),
            shutdown_finished: self.shutdown_finished.clone(),
        }
    }
}

impl std::ops::Drop for ShutdownHandle {
    fn drop(&mut self) {
        debug!("Task terminated");
    }
}
