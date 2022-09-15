// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//!TODO(ppiotr3k): write crate documentation

use async_trait::async_trait;

/// A supertrait to abstract frontend and backend SQL Messages.
// Note: this supertrait will be more valuable when enum variant types are available.
pub trait SQLMessage: Send + Sync {}

/// A trait defining an interface for handling `SQLMessage`s.
///
/// The default implementation is simply a passthrough. An `SQLMessageHandler`
/// is of value when it applies a transformation to the processed `SQLMessage`.
#[async_trait]
pub trait SQLMessageHandler<M>
where
    M: Send + Sync,
{
    ///
    async fn process(&self, msg: M) -> M
    where
        M: SQLMessage + 'async_trait,
    {
        msg
    }
}

//TODO(ppiotr3k): do something about those tests
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
