// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

use bytes::{BufMut, Bytes, BytesMut};
use std::fmt::Debug;

/// A trait defining an interface for data masking strategies.
//TODO(ppiotr3k): consider moving to interfaces
//TODO(ppiotr3k): refactor/closure to dedup logging code
pub trait MaskingStrategy: Debug + Send + Sync {
    fn mask(&self, data: &Bytes) -> Bytes;
}

/// A simple and fast masking strategy where whatever the provided data, the
/// result will be a repetition of `*` characters of requested `length`.
#[derive(Debug)]
pub struct CaviarMask {
    length: usize,
}

impl CaviarMask {
    pub const fn new(length: usize) -> Self {
        Self { length }
    }
}

impl MaskingStrategy for CaviarMask {
    fn mask(&self, data: &Bytes) -> Bytes {
        log::trace!(" original value: {:?}", data);

        let mut res = BytesMut::with_capacity(self.length);
        res.put_bytes(b'*', self.length);

        log::trace!("rewritten value: {:?}", res);
        res.freeze()
    }
}

/// A shape-preserving masking strategy where only alphanumeric characters from
/// provided `data` will be replaced by `*` characters. By preserving shape,
/// this strategy leaks both length and general aspect information.
#[derive(Debug)]
pub struct CaviarShapeMask;

impl CaviarShapeMask {
    pub const fn new() -> Self {
        Self
    }
}

impl MaskingStrategy for CaviarShapeMask {
    //TODO(ppiotr3k): add support for UTF8
    fn mask(&self, data: &Bytes) -> Bytes {
        log::trace!(" original value: {:?}", data);

        let mut res = BytesMut::with_capacity(data.len());
        for c in data.iter() {
            if (*c as char).is_alphanumeric() {
                res.put_u8(b'*');
            } else {
                res.put_u8(*c);
            }
        }

        log::trace!("rewritten value: {:?}", res);
        res.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn valid_caviar_empty_data() {
        let data = Bytes::from_static(b"");
        let expected = Bytes::from_static(b"******");

        let strategy = CaviarMask::new(6);
        let masked = strategy.mask(&data);
        assert_eq!(expected, masked, "masked data");
    }

    #[test]
    fn valid_caviar_single_char_data() {
        let data = Bytes::from_static(b"P");
        let expected = Bytes::from_static(b"******");

        let strategy = CaviarMask::new(6);
        let masked = strategy.mask(&data);
        assert_eq!(expected, masked, "masked data");
    }

    #[test]
    fn valid_caviar_shape_empty_data() {
        let data = Bytes::from_static(b"");
        let expected = Bytes::from_static(b"");

        let strategy = CaviarShapeMask::new();
        let masked = strategy.mask(&data);
        assert_eq!(expected, masked, "masked data");
    }

    #[test]
    fn valid_caviar_shape_single_hyphen_data() {
        let data = Bytes::from_static(b"abcd-efgh");
        let expected = Bytes::from_static(b"****-****");

        let strategy = CaviarShapeMask::new();
        let masked = strategy.mask(&data);
        assert_eq!(expected, masked, "masked data");
    }
}
