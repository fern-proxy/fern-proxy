// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

use tokio::sync::broadcast;

/// Listens for server shutdown signal.
///
/// Uses [`broadcast::Receiver`] for signalling, with a single value ever sent.
/// Once the value has been sent via [`broadcast`], server should shutdown.
///
/// The `Shutdown` struct listens for the signal and tracks signal reception.
/// Callers may query for whether the shutdown signal has been received or not.
///
/// [`broadcast::Receiver`]: https://docs.rs/tokio/*/tokio/sync/broadcast/struct.Receiver.html
/// [`broadcast`]: https://docs.rs/tokio/*/tokio/sync/broadcast/index.html
#[derive(Debug)]
pub(crate) struct Shutdown {
    /// `true` if the shutdown signal has been received already.
    shutdown: bool,

    /// Receiver half of the channel, used to listen for shutdown signal.
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    /// Creates a `Shutdown` instance backed by provided [`broadcast::Receiver`].
    ///
    /// [`broadcast::Receiver`]: https://docs.rs/tokio/*/tokio/sync/broadcast/struct.Receiver.html
    pub(crate) fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            notify,
        }
    }

    /// Returns `true` if the shutdown signal has been received, `false` otherwise.
    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    /// Receives the shutdown signal, waiting if necessary.
    pub(crate) async fn recv(&mut self) {
        // No need to notify about reception nor remember state
        // if the shutdown signal has been received already.
        if self.shutdown {
            return;
        }

        // Wait until the shutdown signal is received.
        // Note: not subject to [`slow receiver`] problem as only one value is ever sent.
        //
        // [`slow receiver`]: https://docs.rs/tokio/*/tokio/sync/broadcast/index.html#lagging
        let _ = self.notify.recv().await;

        // Remember that shutdown signal has been received.
        self.shutdown = true;
    }
}
