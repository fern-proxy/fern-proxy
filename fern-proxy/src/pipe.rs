// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//!TODO(ppiotr3k): write module description

use futures::{sink::SinkExt, stream::StreamExt};
use tokio::io::{AsyncRead, AsyncWrite, Result};
use tokio::sync::mpsc;
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

use fern_proxy_interfaces::{SQLMessage, SQLMessageHandler, SharedConnectionContext};

/// Direction of Messages flow in a `Pipe`.
#[derive(Debug)]
pub enum Direction {
    /// Messages from Client to proxied Server.
    ClientServer,

    /// Messages from proxied Server to Client.
    ServerClient,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ClientServer => write!(f, "Client -> Server"),
            Self::ServerClient => write!(f, "Server -> Client"),
        }
    }
}

///TODO(ppiotr3k): write struct description
#[derive(Debug)]
pub struct Pipe<R, W, C, I, S, H> {
    /// Direction of data flow (Client -> Server; Server -> Client).
    direction: Direction,

    /// [`Stream`] end of the `Pipe`, where data is read from as frames.
    ///
    /// [`Stream`]: https://docs.rs/futures/*/futures/stream/trait.Stream.html
    stream: FramedRead<R, C>,

    /// [`Sink`] end of the `Pipe`, where data is written to as frames.
    ///
    /// [`Sink`]: https://docs.rs/futures/*/futures/sink/trait.Sink.html
    sink: FramedWrite<W, C>,

    /// Chain of `SQLMessageHandler`s applied to data flowing in the `Pipe`.
    /// The chain is built before beeing passed to the `Pipe` constructor.
    //TODO(ppiotr3k): implement a higher-level struct processing the chain automatically
    //TODO(ppiotr3k): settle on the naming: `frame`, `message`, `packet`, ...
    frame_handlers: H,

    /// Access to the `stream` and `sink` of the `Pipe` paired with this one.
    /// Used for "short-circuiting" regular Client <-> proxied Server flows.
    _short_circuit: ShortCircuit<I, S>,
}

impl<R, W, C, I, S, H> Pipe<R, W, C, I, S, H>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
    C: Decoder + Decoder<Item = I> + Encoder<I> + Default,
    I: SQLMessage,
    S: SQLMessage,
    H: SQLMessageHandler<I> + Send + Sync,
{
    ///TODO(ppiotr3k): write function description
    pub fn new(
        direction: Direction,
        receiver: R,
        sender: W,
        short_circuit: ShortCircuit<I, S>,
        ctx: &SharedConnectionContext,
    ) -> Pipe<R, W, C, I, S, H> {
        // Adapt from `AsyncRead`/ `AsyncWrite` to `Stream`/`Sink`.
        Pipe {
            direction,
            stream: FramedRead::new(receiver, C::default()),
            sink: FramedWrite::new(sender, C::default()),
            frame_handlers: H::new(ctx),
            _short_circuit: short_circuit,
        }
    }

    ///TODO(ppiotr3k): write function description
    pub async fn run(&mut self) -> Result<()>
    where
        <C as Encoder<I>>::Error: std::fmt::Display,
        std::io::Error: From<<C as Encoder<I>>::Error>,
    {
        log::trace!("[{}] running pipe", self.direction);

        //TODO(ppiotr3k): investigate if listening for shutdown is required here
        // -> since pipes aren't tasks but infinite loops on futures, seems unnecessary
        loop {
            // `select!` continuously runs all futures until one returns.
            // Read request frame, also listening for the shutdown signal.
            let mut packet = tokio::select! {
                // Await for a Message from `Stream`, or terminate if `Stream` dried.
                result = self.stream.next() => {
                    if let Some(Ok(packet)) = result {
                        packet
                    } else {
                        let err = std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            format!("[{}] read 0 bytes, closing pipe", self.direction),
                        );
                        log::trace!("{}", err);
                        return Err(err);
                    }
                },

                // // Process a short-circuit Message.
                // result = self.short_circuit.rx.recv() => {
                //     if let Some(packet) = result {
                //         log::trace!(
                //             "[{}] received short-circuit packet: {:?}",
                //             self.direction,
                //             packet,
                //         );
                //         packet
                //     } else {
                //         let err = std::io::Error::new(
                //             std::io::ErrorKind::UnexpectedEof,
                //             format!(
                //                 "[{}] paired pipe prematurely closed",
                //                 self.direction
                //             )
                //         );
                //         log::trace!("{}", err);
                //         return Err(err)
                //     }
                // },
            };

            //TODO(ppiotr3k): check if `packet` should be short-circuited
            //TODO(ppiotr3k): process `packet` through "packet handlers"

            packet = self.frame_handlers.process(packet).await;

            //TODO(ppiotr3k): consider batching rather than `send`ing one-by-one
            // Write `packet` to `Sink`, and flush it.
            if let Err(err) = self.sink.send(packet).await {
                log::error!("[{}] cannot send to sink: {}", self.direction, err);
                return Err(err.into());
            }
        }
    }
}

///TODO(ppiotr3k): write struct description
#[derive(Debug)]
pub struct ShortCircuit<R, S> {
    _tx: mpsc::Sender<S>,
    _rx: mpsc::Receiver<R>,
}

impl<R, S> ShortCircuit<R, S> {
    pub fn new(tx: mpsc::Sender<S>, rx: mpsc::Receiver<R>) -> ShortCircuit<R, S> {
        ShortCircuit { _tx: tx, _rx: rx }
    }
}
