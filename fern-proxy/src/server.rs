// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};

use crate::connection::Connection;
use crate::shutdown::Shutdown;

/// Maximum number of concurrent connections the listener will accept.
///
/// When this limit is reached, the listener stops accepting connections
/// until an active connection terminates.
//TODO(ppiotr3k): make this value configurable
// Set to '1' to test permit acquisition.
const MAX_CONNECTIONS: usize = 1;

/// Server listener state, created with `server::run`.
/// Performs TCP listening and initialization of per-connection state.
//TODO(ppiotr3k): enhance struct description
#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    proxied_server: String,
    notify_shutdown: broadcast::Sender<()>,
    limit_connections: Arc<Semaphore>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

/// Per-connection handler.
//TODO(ppiotr3k): enhance struct description
#[derive(Debug)]
struct Handler {
    ///TODO(ppiotr3k): write description
    connection: Connection,

    /// Future listenning for shutdown notifications.
    shutdown: Shutdown,

    ///TODO(ppiotr3k): write description
    _shutdown_complete: tokio::sync::mpsc::Sender<()>,
}

impl Handler {
    /// Process a single inbound connection.
    ///
    /// Continuously runs forward/backward pipes for a single connection,
    /// to handle Messages flows between Client and proxied Server.
    ///
    /// When the shutdown signal is received, first the processing
    /// continues until a safe state is reached, then it is terminated.
    async fn run(&mut self) -> crate::Result<()> {
        // Process frames, also listening for the shutdown signal.
        while !self.shutdown.is_shutdown() {
            // `select!` continuously runs all futures until one returns.
            // Note : pipes are infinite loops; they never exit unless an error happens.
            log::trace!("starting forward/backward pipes");
            tokio::select! {
                _ = self.connection.forward_pipe.run() => {
                    log::trace!("pipe closed via forward pipe");
                    break;
                },
                _ = self.connection.backward_pipe.run() => {
                    log::trace!("pipe closed via backward pipe");
                    let err = std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "remote server prematurely closed connection"
                    );
                    // Server closed connection, task must be terminated.
                    return Err(err);
                },
                _ = self.shutdown.recv() => {
                    log::trace!("pipe closed via shutdown signal");
                },
            }
        }
        // Client closed connection, or shutdown signal has been received.
        Ok(())
    }
}

impl Listener {
    /// Runs the PostgreSQL proxy.
    ///
    /// Listens for inbound connections, and for each one spawns
    /// a `Handler` task to manage that connection independently.
    ///
    /// # Errors
    ///
    /// Returns `Err` if accepting returns an error, which can happen
    /// for multiple reasons like the underlying operating system having
    /// reached an internal limit for max number of sockets in use.
    ///
    /// Those kind of errors resolving by themselves over time, yet the process
    /// not being able to detect resolution, a backoff strategy is implemented.
    pub async fn run(&mut self) -> crate::Result<()> {
        log::info!("listener is running, awaiting connections");
        loop {
            // Await for a `SemaphorePermit` to become available.
            // Note: `acquire_owned` returns a permit that is bound to the semaphore.
            // The permit is automatically returned to the semaphore once dropped.
            // Note: `acquire_owned` returns an `Err` if the semaphore has been
            // closed. As the semaphore is never closed, it is safe to `unwrap`.
            log::trace!("awaiting permit to accept new connection");
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            log::trace!(
                "permit acquired (remaining: {})",
                self.limit_connections.available_permits()
            );

            //TODO(ppiotr3k): investigate potential blocking condition(s) for shutdown signal to pass
            log::debug!("awaiting new connection or shutdown signal");

            // Accept a new socket, attempting to perform error handling.
            // Note: `accept` attempts internally to recover from errors,
            // therefore an error returned by `accept` is non-recoverable.
            let (client_socket, client_addr) = self.accept().await?;
            log::info!("new connection from: {}", client_addr);

            // Create connection with proxied Server.
            //TODO(ppiotr3k): define expected behaviour when proxied server is not available
            // -> (fail? do not accept connection? etc.)
            //TODO(ppiotr3k): consider using server connection pool
            let server_socket = TcpStream::connect(&self.proxied_server)
                .await
                .expect("error - failed connecting to proxied server");

            // let server_socket =
            //     match TcpStream::connect(self.proxied_server).await {
            //         Ok(socket) => socket,
            //         _ => {
            //             log::error!("failed connecting to proxied server");
            //             break;
            //         },
            //     };

            // Initialize per-connection handler state.
            let mut handler = Handler {
                // Initialize connection state (buffered wrapper for `TcpStream`).
                connection: Connection::new(client_socket, server_socket).await,

                // Receive shutdown notification.
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),

                // Notify receiver half once are clones are dropped.
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            // Spawn a new concurrent task to process the connection.
            tokio::spawn(async move {
                // Process the connection, log any error.
                log::trace!("spawned task to manage {}", client_addr);
                if let Err(err) = handler.run().await {
                    log::error!("connection error: {}", err);
                }

                // Return permit to semaphore once task completed.
                drop(permit);
                log::info!("closing connection from: {}", client_addr);
            });
        }
    }

    /// Accepts an inbound Client connection.
    ///
    /// An exponential backoff strategy is used to handle errors, until a limit
    /// is reached, and then the operation is aborted. In the implemented strategy,
    /// after 1st failure, the task waits for 1 second. Then, after the 2nd failure,
    /// the wait is 2 seconds. Each subsequent failure doubles the wait time.
    ///
    /// # Errors
    ///
    /// After the 6th attempt, which is 64 seconds after the 5th attempt, if
    /// accepting is still failing, then this function aborts, returning with an error.
    async fn accept(&mut self) -> crate::Result<(TcpStream, std::net::SocketAddr)> {
        let mut backoff = 1;

        // Try accepting up to 6 times, with an exponential wait in-between.
        loop {
            match self.listener.accept().await {
                Ok((socket, peer_addr)) => return Ok((socket, peer_addr)),
                Err(err) => {
                    if backoff > 64 {
                        // `accept` has failed too many times.
                        log::trace!("accept failed too many times, backoff strategy exhausted");
                        return Err(err);
                    }
                }
            }

            // Pause execution until the back off period elapses.
            tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;

            // Double the time before next attempt.
            backoff *= 2;
        }
    }
}

/// Runs the server.
///
/// Accepts connections from the supplied listener, and for each inbound
/// connection a task is spawned to handle that connection. The server runs
/// indefinitely, until either a shutdown signal is received, or the proxied Server
/// terminates the connection, at which point the server shuts down gracefully.
pub async fn run(listener: TcpListener, srv_addr: &str, shutdown: impl Future) {
    // When the provided `shutdown` future completes, i.e. shutdown signal is
    // received, the shutdown signal must be propagated to to all active connections.
    // This is implemented using a broadcast channel where only 1 message will be ever sent.
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = tokio::sync::mpsc::channel(1);

    // Initialize listener state.
    let mut server = Listener {
        listener,
        //TODO(ppiotr3k): investigate avoidable memory alloc
        proxied_server: srv_addr.to_string(),
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_rx,
        shutdown_complete_tx,
    };

    // Infinite loop, unless a critical error or shutdown signal is encountered.
    tokio::select! {
        result = server.run() => {
            // If an error is received here, this means that accepting
            // connections from the TCP listener failed multiple times,
            // and that the server is giving up and shutting down.
            // No errors from per-connection handlers bubble up to here.
            if let Err(err) = result {
                log::error!("failed to accept: {}", err);
            }
        },

        _ = shutdown => {
            // Shutdown signal has been received.
            log::info!("shutdown signal received; shutting down listener");
        }
    }

    // Extract specific fields to explicitely drop them.
    // Note: otheriwise the `.await` below would never complete.
    let Listener {
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    // When dropping `notify_shutdown`, all tasks which have
    // `subscribe`d will receive the shutdown signal and can terminate.
    drop(notify_shutdown);
    // Drop final `Sender` so the `Receiver` below can complete.
    drop(shutdown_complete_tx);

    // Shut down gracefully, waiting for all active connections to finish
    // processing by returning to a safe state. As the `Sender` handle held
    // by this listener has been dropped above, the only remaining `Sender`
    // instances are held by connection handler tasks. When those handler tasks
    // drop, the `mpsc` channel will close, and `recv` will return `None`.
    let _ = shutdown_complete_rx.recv().await;
}
