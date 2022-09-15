// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//! # PostgreSQL wire protocol
//!
//! Structs and functions for implementing the PostgreSQL wire protocol
//! [Message Formats], as needed by Fern proxy.
//!
//!
//! ## Disclaimer
//! By no means does this crate aim to implement Structs for PostgreSQL
//! messages that could be used for building general purpose or specialized
//! PostgreSQL clients and/or servers. Structs in this crate aim only to
//! support the specific needs of the Fern proxy project.
//!
//! ## Examples
//!
//! ```rust
//! use fern_protocol_postgres::codec::frontend::{Codec, Message};
//! ```
//!
//! [Message Formats]: https://www.postgresql.org/docs/current/protocol-message-formats.html

#![forbid(unsafe_code)]

pub mod codec;
