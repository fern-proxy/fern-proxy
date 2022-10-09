// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//!TODO(ppiotr3k): write crate documentation

use async_trait::async_trait;
use config::Config;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

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

    fn new(context: &SharedConnectionContext) -> Self
    where
        Self: Sized;
}

/// A struct to store and share information local to a `Connection` instance.
///
/// Each `Connection` being managed independently by a `Handler`, which itself
/// is spawned in a [`tokio::task`], a `ConnectionContext` instance requires
/// using `Sync` and `Send` primitives to safely read and/or write data.
///
/// Since read operations on a `ConnectionContext` instance are more frequent
/// than write operations, an [`RwLock`] has been preferred to allow multiple
/// readers at the same time, while allowing only one writer at a time.
///
/// Instantiating with `new` returns a `SharedConnectionContext` which simply
/// is a type alias for `Arc<RwLock<ConnectionContext>>`.
///
/// [`tokio::task`]: https://docs.rs/tokio/latest/tokio/task/
/// [`RwLock`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html
#[derive(Debug)]
pub struct ConnectionContext {
    /// Configuration data required to instantiate `Connection`.
    pub config: Config,

    /// General purpose key-value store of `String` type.
    pub store: HashMap<String, String>,
}

/// A type alias for `ConnectionContext` with synchronization primitives.
pub type SharedConnectionContext = Arc<RwLock<ConnectionContext>>;

impl ConnectionContext {
    pub fn new(config: &Config) -> SharedConnectionContext {
        let context = Self {
            config: config.clone(),
            store: HashMap::new(),
        };
        Arc::new(RwLock::new(context))
    }
}

//TODO(ppiotr3k): do something about those tests
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
