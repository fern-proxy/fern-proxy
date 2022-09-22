// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::pipe::{Direction, Pipe, ShortCircuit};
use fern_masking::{DataMaskingHandler, PassthroughHandler, SQLHandlerConfig};
use fern_protocol_postgresql::codec::{backend, frontend};

//TODO(ppiotr3k): write description
#[derive(Debug)]
pub struct Connection {
    /// `Pipe` instance processing Messages from Client to proxied Server.
    pub forward_pipe: Pipe<
        OwnedReadHalf,
        OwnedWriteHalf,
        frontend::Codec,
        frontend::Message,
        backend::Message,
        PassthroughHandler<frontend::Message>,
    >,

    /// `Pipe` instance processing Messages from proxied Server to Client.
    pub backward_pipe: Pipe<
        OwnedReadHalf,
        OwnedWriteHalf,
        backend::Codec,
        backend::Message,
        frontend::Message,
        DataMaskingHandler,
    >,
}

impl Connection {
    /// Creates a new connection for proxying provided `client_socket` and `server_socket`.
    #[rustfmt::skip]
    pub async fn new(client_socket: TcpStream, server_socket: TcpStream, config: &SQLHandlerConfig) -> Connection {
        // Split the sockets to be able to `Pipe` them together.
        let (client_rx, client_tx) = client_socket.into_split();
        let (server_rx, server_tx) = server_socket.into_split();

        // Create channels to allow short-circuiting regular Message flows.
        let (forward_tx, forward_rx) = mpsc::channel::<backend::Message>(128);
        let (backward_tx, backward_rx) = mpsc::channel::<frontend::Message>(128);
        let forward_short = ShortCircuit::new(forward_tx, backward_rx);
        let backward_short = ShortCircuit::new(backward_tx, forward_rx);

        // Create `Pipe` instance for regular Client -> proxied Server Message flows.
        let forward_pipe = Pipe::new(
            Direction::ClientServer,
            client_rx,
            server_tx,
            forward_short,
            config,
        );

        // Create `Pipe` instance for regular proxied Server -> Client Message flows.
        let backward_pipe = Pipe::new(
            Direction::ServerClient,
            server_rx,
            client_tx,
            backward_short,
            config,
        );

        Connection {
            forward_pipe,
            backward_pipe,
        }
    }
}
