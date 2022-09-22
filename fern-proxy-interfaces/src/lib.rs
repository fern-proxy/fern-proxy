// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//!TODO(ppiotr3k): write crate documentation

use async_trait::async_trait;
use std::fmt::Debug;

// Re-export.
//TODO(ppiotr3k): consider a "scoped" config for Handler needs
pub use config::Config as SQLHandlerConfig;

/// A supertrait to abstract frontend and backend SQL Messages.
// Note: this supertrait will be more valuable when enum variant types are available.
pub trait SQLMessage: Debug + Send + Sync {}

/// A trait defining an interface for handling `SQLMessage`s.
///
/// The default implementation is simply a passthrough. An `SQLMessageHandler`
/// is of value when it applies a transformation to the processed `SQLMessage`.
#[async_trait]
pub trait SQLMessageHandler<M>: Debug + Send + Sync
where
    M: Send + Sync,
{
    /// Applies a transformation to an `SQLMessage`.
    async fn process(&mut self, msg: M) -> M
    where
        M: SQLMessage + 'async_trait,
    {
        msg
    }

    fn new(config: &SQLHandlerConfig) -> Self
    where
        Self: Sized;
}

//TODO(ppiotr3k): do something about those tests
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
