// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//! Adaptors from [`AsyncRead`]/[`AsyncWrite`] to [`Stream`]/[`Sink`],
//! decoding/encoding PostgreSQL protocol version 3.
//!
//! [`AsyncRead`]: https://docs.rs/tokio/*/tokio/io/trait.AsyncRead.html
//! [`AsyncWrite`]: https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html
//! [`Stream`]: https://docs.rs/futures/*/futures/stream/trait.Stream.html
//! [`Sink`]: https://docs.rs/futures/*/futures/sink/trait.Sink.html

use fern_proxy_interfaces::SQLMessage;

pub mod backend;
pub mod frontend;

/// A trait to abstract frontend and backend Messages.
//TODO(ppiotr3k): make private again
pub trait PostgresMessage: SQLMessage + std::fmt::Debug {}

/// Collection of constants used internally.
pub(crate) mod constants {
    /// Amount of bytes for PostreSQL Message identifier.
    pub const BYTES_MESSAGE_ID: usize = 1;

    /// Amount of bytes for PostreSQL Message payload size, including self.
    pub const BYTES_MESSAGE_SIZE: usize = 4;

    /// Amount of bytes for PostreSQL Message header.
    pub const BYTES_MESSAGE_HEADER: usize = BYTES_MESSAGE_ID + BYTES_MESSAGE_SIZE;
}

/// Collection of bytes manipulating helper functions used internally.
pub(crate) mod utils {

    use bytes::{Buf, BufMut, Bytes, BytesMut};
    use std::io;

    /// Creates a new [`Bytes`] instance by first reading a length header of
    /// `length_bytes`, and then getting as many bytes.
    ///
    /// The current position in `buf` is advanced by `length_bytes` and
    /// the value contained in the size header.
    ///
    /// This function is optimized to avoid copies by using a [`Bytes`]
    /// implementation which only performs a shallow copy.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] with `error_msg` if there is not enough data.
    ///
    /// [`Bytes`]: https://docs.rs/bytes/*/bytes/struct.Bytes.html
    pub(crate) fn get_bytes(
        buf: &mut BytesMut,
        length_bytes: usize,
        error_msg: &str,
    ) -> io::Result<Bytes> {
        // Shouldn't happend, unless packet is malformed.
        if buf.remaining() < length_bytes {
            let err = io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "malformed packet - invalid data size",
            );
            log::error!("{}", err);
            return Err(err);
        }

        let data_length = buf.get_u32();
        log::trace!("bytes data length header: {}", data_length);

        let data = if data_length == u32::MAX {
            Bytes::new()
        } else {
            if buf.remaining() < data_length as usize {
                log::error!("{}", error_msg);
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, error_msg));
            }
            buf.copy_to_bytes(data_length as usize)
        };

        Ok(data)
    }

    /// Writes bytes to `buf`, prefixing `data` with a size header.
    ///
    /// The current position in `buf` is advanced by the length of `data`,
    /// and the 4 bytes required to define the size header.
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in `buf`.
    pub(crate) fn put_bytes(data: &Bytes, buf: &mut BytesMut) {
        buf.put_u32(data.len() as u32);
        // Cloning `Bytes` is cheap, it is an `Arc` increment.
        buf.put((*data).clone());
    }

    /// Gets a C-style null character terminated string from `buf`
    /// as a new [`Bytes`] instance.
    ///
    /// The current position in `buf` is advanced by the length of the string,
    /// and 1 byte to account for the null character terminator.
    ///
    /// This function is optimized to avoid copies by using a [`Bytes`]
    /// implementation which only performs a shallow copy.
    ///
    /// Returns [`io::ErrorKind::InvalidData`] if no null chararacter terminator if found.
    ///
    /// [`Bytes`]: https://docs.rs/bytes/*/bytes/struct.Bytes.html
    pub(crate) fn get_cstr(buf: &mut BytesMut) -> io::Result<Bytes> {
        let nullchar_offset = buf[..].iter().position(|x| *x == b'\0');

        match nullchar_offset {
            None => {
                let err = io::Error::new(
                    io::ErrorKind::InvalidData,
                    "malformed packet - cstr without null char terminator",
                );
                log::error!("{}", err);
                Err(err)
            }
            Some(offset) => {
                let str_bytes = buf.copy_to_bytes(offset);
                buf.advance(1); // consume null char
                Ok(str_bytes)
            }
        }
    }

    /// Writes a C-style null character terminated string to `buf`.
    ///
    /// The current position in `buf` is advanced by the length of the string,
    /// and 1 byte to account for the null character terminator.
    ///
    /// # Note
    ///
    /// The [`Bytes`] instance defining the string is deemed to contain
    /// only valid ASCII characters, as it is deemed to originate from a
    /// controlled and/or sanitized context.
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in `buf`.
    ///
    /// [`Bytes`]: https://docs.rs/bytes/*/bytes/struct.Bytes.html
    pub(crate) fn put_cstr(string: &Bytes, buf: &mut BytesMut) {
        // Cloning `Bytes` is cheap, it is an `Arc` increment.
        buf.put((*string).clone());
        buf.put_u8(b'\0');
    }

    /// Gets an unsigned 32-bit integer from `buf` in big-endian byte order,
    /// and advances current position by 4.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] with `error_msg` if there is not enough data.
    pub(crate) fn get_u32(buf: &mut BytesMut, error_msg: &str) -> io::Result<u32> {
        if buf.remaining() < 4 {
            log::error!("{}", error_msg);
            return Err(io::Error::new(io::ErrorKind::InvalidData, error_msg));
        }

        let value = buf.get_u32();
        Ok(value)
    }

    /// Gets a signed 32-bit integer from `buf` in big-endian byte order,
    /// and advances current position by 4.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] with `error_msg` if there is not enough data.
    pub(crate) fn get_i32(buf: &mut BytesMut, error_msg: &str) -> io::Result<i32> {
        if buf.remaining() < 4 {
            log::error!("{}", error_msg);
            return Err(io::Error::new(io::ErrorKind::InvalidData, error_msg));
        }

        let value = buf.get_i32();
        Ok(value)
    }

    /// Gets an unsigned 16-bit integer from `buf` in big-endian byte order,
    /// and advances current position by 2.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] with `error_msg` if there is not enough data.
    pub(crate) fn get_u16(buf: &mut BytesMut, error_msg: &str) -> io::Result<u16> {
        if buf.remaining() < 2 {
            log::error!("{}", error_msg);
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, error_msg));
        }

        let value = buf.get_u16();
        Ok(value)
    }

    /// Gets a signed 16-bit integer from `buf` in big-endian byte order,
    /// and advances current position by 2.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] with `error_msg` if there is not enough data.
    pub(crate) fn get_i16(buf: &mut BytesMut, error_msg: &str) -> io::Result<i16> {
        if buf.remaining() < 2 {
            log::error!("{}", error_msg);
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, error_msg));
        }

        let value = buf.get_i16();
        Ok(value)
    }

    /// Gets an unsigned 8-bit integer from `buf` in big-endian byte order,
    /// and advances current position by 1.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] with `error_msg` if there is not enough data.
    pub(crate) fn get_u8(buf: &mut BytesMut, error_msg: &str) -> io::Result<u8> {
        if buf.remaining() < 1 {
            log::error!("{}", error_msg);
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, error_msg));
        }

        let value = buf.get_u8();
        Ok(value)
    }
}
