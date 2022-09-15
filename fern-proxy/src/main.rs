// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

use tokio::{io::Result, net::TcpListener};

mod connection;
mod pipe;
mod server;
mod shutdown;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Per "12 factors: III. Config", store config in the environment.
    let own_addr = std::env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0:30000".into());
    log::trace!("listener addr: {}", own_addr);

    let srv_addr = std::env::var("SERVER").expect("SERVER env variable is undefinied");
    log::trace!("proxied Server addr: {}", srv_addr);

    //TODO(ppiotr3k): support instanciation of multiple listener tasks
    //TODO(ppiotr3k): consider multiple processes and CPU affinity
    let listener = TcpListener::bind(own_addr).await?;

    // Run until `<CTRL> + C` is hit - equivalent to SIGINT signal.
    server::run(listener, &srv_addr, tokio::signal::ctrl_c()).await;
    log::info!("proxy shut down; exiting");
    Ok(())
}
