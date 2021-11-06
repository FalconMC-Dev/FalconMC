use tokio::sync::{broadcast, mpsc};

/// A handle to help asynchronously shutting down our application
///
/// # Safety
/// This handler can safely be cloned, it is even the way you're supposed
/// to pass this handle to other parts of the program. Internally it uses
/// two channels: one for sending a shutdown signal to all clones,
/// and one it uses as a marker to signal a "shutdown complete from this part of the program" when the handle goes out of scope.
pub struct ShutdownHandle {
    #[doc(hidden)]
    signal_sender: broadcast::Sender<()>,
    #[doc(hidden)]
    signal_receiver: broadcast::Receiver<()>,
    #[doc(hidden)]
    shutdown_finished: mpsc::Sender<()>,
}

impl ShutdownHandle {
    /// Returns a Shutdown handle with the sender ("hook") given.
    ///
    /// # Arguments
    ///
    /// * `finished_hook` - A Tokio sender from which the corresponding receiver can be used
    /// to know whether the process owning this handle is finished.
    ///
    /// # Note
    /// This handle is also able to signal all other clones of this handle
    /// when the program wants to shut down in order for necessary cleanup to happen.
    ///
    /// More info under [`Self::send_shutdown()`] and [`Self::wait_for_shutdown()`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tokio::sync::mpsc::channel;
    /// use std::mem::drop;
    ///
    /// use limbo_core::ShutdownHandle;
    ///
    /// async fn process() {
    ///     let (send, mut recv) = channel(1);
    ///     let shutdown_handle = ShutdownHandle::new(send);
    ///
    ///     // Pass this handle through cloning to other parts of the program that should be cleaned up when shutting down
    ///
    ///     // Drop the local scope handle before waiting
    ///     drop(shutdown_handle);
    ///     let _ = recv.recv().await;
    ///     // everything has shut down
    /// }
    /// ```
    pub fn new(finished_hook: mpsc::Sender<()>) -> ShutdownHandle {
        let (sender, receiver) = broadcast::channel(1);
        ShutdownHandle {
            signal_sender: sender,
            signal_receiver: receiver,
            shutdown_finished: finished_hook,
        }
    }

    /// Signals all clones of this Handle to initiate shutdown.
    ///
    /// When the owner of this handle has finished, it should make sure to have this handle go out of scope.
    ///
    /// # Examples
    /// ```ignore
    /// use tokio::sync::mpsc::channel;
    /// use std::mem::drop;
    ///
    /// use limbo_core::ShutdownHandle;
    ///
    /// fn do_something() {
    ///     let (send, mut recv) = channel(1);
    ///     let shutdown_handle = ShutdownHandle::new(send);
    ///
    ///     // Pass this handle through cloning to all other parts of the program that should know when to shut down
    ///
    ///     // when we want to shutdown
    ///     shutdown_handle.send_shutdown();
    ///
    ///     // now we can wait for everything to have shutdown as explained in `ShutdownHandle::new()`
    /// }
    pub fn send_shutdown(&self) {
        let _ = self.signal_sender.send(());
    }

    /// Polls for the status of this handle's signal status.
    /// # Returns
    /// True when a shutdown signal was received.
    ///
    /// False when a shutdown signal was received.
    /// # Note
    /// This function does not block.
    pub fn try_is_shutdown(&mut self) -> bool {
        if let Err(error) = self.signal_receiver.try_recv() {
            if error == broadcast::error::TryRecvError::Empty {
                return false;
            }
        }
        true
    }

    /// Waits for this handle to receive a shutdown signal and returns.
    /// # Note
    /// Use this function to know when to trigger the shutdown sequence of its owner.
    ///
    /// This function asynchronously waits before returning
    pub async fn wait_for_shutdown(&mut self) -> Result<(), broadcast::error::RecvError> {
        self.signal_receiver.recv().await
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
        debug!("Dropping handle!");
    }
}