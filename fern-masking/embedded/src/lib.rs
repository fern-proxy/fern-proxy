// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use bytes::Bytes;

use fern_protocol_postgresql::codec::backend;
use fern_proxy_interfaces::{SQLMessage, SQLMessageHandler, SharedConnectionContext};

use crate::strategies::MaskingStrategy;

/// An `SQLMessageHandler` applying a data masking strategy.
///
/// The [`MaskingStrategy`] to use is set at struct instantiation,
/// depending on settings in provided `SQLHandlerConfig`.
///
/// Should no settings be defined for data masking, by default a
/// fixed-length caviar strategy will mask all `DataRow` fields.
#[derive(Debug)]
pub struct DataMaskingHandler {
    state: QueryState,

    /// Masking strategy applied by this Handler.
    //TODO(ppiotr3k): investigate if `Box`-ing can be avoided
    strategy: Box<dyn MaskingStrategy>,

    /// Column names where masking will not be applied, unless forced.
    columns_excluded: Vec<Bytes>,

    /// Column names where masking will be applied, in any case.
    /// This allows using a wildcard in exclusions, and progressively mask.
    columns_forced: Vec<Bytes>,
}

///TODO(ppiotr3k): write description
#[derive(Debug)]
enum QueryState {
    /// Awaiting for a `RowDescription` Message.
    Description,

    /// Processing `DataRow` Messages.
    Data(Vec<usize>),
}

//TODO(ppiotr3k): this crate should only process abstracted types
#[async_trait]
impl SQLMessageHandler<backend::Message> for DataMaskingHandler {
    fn new(context: &SharedConnectionContext) -> Self {
        // Acquire a reader lock on `context`.
        let ctx = context.read().unwrap();

        //TODO(ppiotr3k): make length configurable
        let strategy: Box<dyn MaskingStrategy> =
            if let Ok(strategy) = ctx.config.get::<String>("masking.strategy") {
                match strategy.as_str() {
                    "caviar" => Box::new(strategies::CaviarMask::new(6)),
                    "caviar-preserve-shape" => Box::new(strategies::CaviarShapeMask::new()),
                    _ => Box::new(strategies::CaviarMask::new(6)),
                }
            } else {
                // Default strategy, if nothing is defined in `config`.
                Box::new(strategies::CaviarMask::new(6))
            };

        let mut columns_excluded = vec![];
        if let Ok(columns) = ctx.config.get::<Vec<String>>("masking.exclude.columns") {
            for column_name in columns.iter() {
                columns_excluded.push(Bytes::from(column_name.clone()));
            }
        }

        let mut columns_forced = vec![];
        if let Ok(columns) = ctx.config.get::<Vec<String>>("masking.force.columns") {
            for column_name in columns.iter() {
                columns_forced.push(Bytes::from(column_name.clone()));
            }
        }

        Self {
            state: QueryState::Description,
            strategy,
            columns_excluded,
            columns_forced,
        }
    }

    async fn process(&mut self, msg: backend::Message) -> backend::Message {
        match msg {
            backend::Message::RowDescription(descriptions) => {
                // Define indexes of columns to exclude from masking.
                let mut no_mask = vec![];
                if self.columns_excluded.len() == 1 && self.columns_excluded[0] == "*" {
                    // Wildcard `*` translates to all indexes.
                    // Note: `forced` columns prevail on exclusions anyway.
                    for (idx, description) in descriptions.iter().enumerate() {
                        if !self.columns_forced.contains(&description.name) {
                            no_mask.push(idx);
                        }
                    }
                } else {
                    // If no wildcard, capture columns to exclude from masking.
                    // Note: `forced` columns prevail on exclusions anyway.
                    for (idx, description) in descriptions.iter().enumerate() {
                        if self.columns_excluded.contains(&description.name)
                            && !self.columns_forced.contains(&description.name)
                        {
                            no_mask.push(idx);
                        }
                    }
                }

                // Store indexes to exclude from masking in upcoming `DataRow`s.
                self.state = QueryState::Data(no_mask);
                log::debug!("new masking exclusion state: {:?}", self.state);
                backend::Message::RowDescription(descriptions)
            }
            backend::Message::CommandComplete(command) => {
                // No more `DataRow`s to process, reset state.
                self.state = QueryState::Description;
                log::debug!("resetting masking state, awaiting next query");
                backend::Message::CommandComplete(command)
            }
            backend::Message::DataRow(fields) => {
                log::trace!("processing fields: {:?}", fields);
                let mask = if let QueryState::Data(mask) = &self.state {
                    mask
                } else {
                    panic!("unexpected state for `QueryState`");
                };

                let mut replaced_fields = vec![];
                for (idx, field) in fields.iter().enumerate() {
                    if !mask.contains(&idx) {
                        log::debug!("applying masking to field #{}", idx);
                        let rewritten = self.strategy.mask(field);
                        replaced_fields.push(rewritten);
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

/// Handler used currently for PostgreSQL frontend Messages.
/// Does nothing but passthrough.
#[derive(Debug)]
pub struct PassthroughHandler<M> {
    _phantom: std::marker::PhantomData<M>,
}

#[async_trait]
impl<M> SQLMessageHandler<M> for PassthroughHandler<M>
where
    M: SQLMessage,
{
    fn new(_context: &SharedConnectionContext) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

mod strategies;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
