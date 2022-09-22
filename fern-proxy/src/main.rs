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

    //TODO(ppiotr3k): refactor and move config initialization out of `main`
    // Get settings defined in `CONFIG_FILE`.
    //TODO(ppiotr3k): think how to materialize config as env vars for 12F
    let toml_config = {
        if let Ok(config_file) = std::env::var("CONFIG_FILE") {
            log::debug!("loading config file: '{}'", config_file);
            let config = config::Config::builder()
                .add_source(config::File::new(&config_file, config::FileFormat::Toml))
                .build();
            match config {
                Ok(config) => config,
                Err(err) => {
                    log::error!("aborting - {}", err);
                    //FIXME(ppiotr3k): return proper error code
                    return Ok(());
                }
            }
        } else {
            // No `CONFIG_FILE` defined results in an empty `Config`.
            config::Config::builder().build().unwrap()
        }
    };

    // Merge defaults with settings from `CONFIG_FILE`.
    let config = config::Config::builder()
        .set_default("masking.exclude.columns", "[]")
        //TODO(ppiotr3k): investigate if defaults can actually fail
        .unwrap()
        .add_source(toml_config)
        .build()
        .expect("conflict in config defaults - should not happen");
    log::trace!("using config: {:?}", config);

    //TODO(ppiotr3k): support instanciation of multiple listener tasks
    //TODO(ppiotr3k): consider multiple processes and CPU affinity
    let listener = TcpListener::bind(own_addr).await?;

    // Run until `<CTRL> + C` is hit - equivalent to SIGINT signal.
    server::run(listener, &srv_addr, tokio::signal::ctrl_c(), &config).await;
    log::info!("proxy shut down; exiting");
    Ok(())
}
