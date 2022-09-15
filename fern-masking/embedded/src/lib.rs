// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use bytes::BytesMut;

use fern_protocol_postgresql::codec::{backend, frontend};
use fern_proxy_interfaces::SQLMessageHandler;

/// A dummy data masking Handler implemented only for demo purposes.
#[derive(Debug, Default)]
pub struct DummyHandler {}

/// Dummy Handler for PostgreSQL backend Messages implemented for demo purposes.
///
/// This Handler applies a dummy data masking, replacing every 'o' with an '*'.
/// It is kind of a worse case scenario as the masking is applied to every single
/// field in each row, without filtering on column names by choice, parsing every
/// byte of every DataRow returned as response, etc. Really terrible design. :-)
//TODO(ppiotr3k): this crate should only process abstracted types
#[async_trait]
impl SQLMessageHandler<backend::Message> for DummyHandler {
    async fn process(&self, msg: backend::Message) -> backend::Message {
        match msg {
            backend::Message::DataRow(fields) => {
                log::trace!("!! rewriting fields: {:?}", fields);
                let mut replaced_fields = vec![];
                for field in fields.iter() {
                    if !field.is_empty() {
                        log::trace!("!! original field: {:?}", field);
                        let mut f = BytesMut::with_capacity(field.len());
                        f.extend_from_slice(&field[..]);
                        let mut i = 0;
                        //TODO(ppiotr3k): think how to make this worse but realistic
                        while i < f.len() {
                            if f[i] == b'o' {
                                f[i] = b'*';
                            }
                            i += 1;
                        }
                        let rewriten = f.freeze();
                        log::trace!("!! rewriten field: {:?}", rewriten);
                        replaced_fields.push(rewriten);
                    } else {
                        replaced_fields.push(field.clone());
                    }
                }
                backend::Message::DataRow(replaced_fields)
            }
            _ => msg,
        }
    }
}

/// Dummy Handler for PostgreSQL frontend Messages. Does nothing but passthrough.
//TODO(ppiotr3k): refactor trait/generics so passthrough is available by default
#[async_trait]
impl SQLMessageHandler<frontend::Message> for DummyHandler {
    async fn process(&self, msg: frontend::Message) -> frontend::Message {
        msg
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
