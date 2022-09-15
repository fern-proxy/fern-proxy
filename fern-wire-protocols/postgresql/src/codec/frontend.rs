// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//! [`Decoder`]/[`Encoder`] traits implementations
//! for PostgreSQL frontend Messages.
//!
//! [`Decoder`]: https://docs.rs/tokio-util/*/tokio_util/codec/trait.Decoder.html
//! [`Encoder`]: https://docs.rs/tokio-util/*/tokio_util/codec/trait.Encoder.html

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

use super::{PostgresMessage, SQLMessage};
use crate::codec::constants::*;
use crate::codec::utils::*;

const BYTES_STARTUP_MESSAGE_HEADER: usize = 8;
const MESSAGE_ID_SSL_REQUEST: i32 = 80877103;
const MESSAGE_ID_STARTUP_MESSAGE: i32 = 196608;

// const MESSAGE_ID_BIND: u8 = b'B'; //TODO(ppiotr3k): write tests
const MESSAGE_ID_EXECUTE: u8 = b'E';
const MESSAGE_ID_FLUSH: u8 = b'H';
const MESSAGE_ID_QUERY: u8 = b'Q';
const MESSAGE_ID_SASL: u8 = b'p';
const MESSAGE_ID_SYNC: u8 = b'S';
const MESSAGE_ID_TERMINATE: u8 = b'X';

// TODO(ppiotr3k): implement following messages
// const MESSAGE_ID_CANCEL_REQUEST: u8 = b''; // ! no id; maybe MSB will do //TODO(ppiotr3k): write tests
// const MESSAGE_ID_CLOSE: u8 = b'C'; //TODO(ppiotr3k): write tests
// const MESSAGE_ID_COPY_DATA: u8 = b'd'; //TODO(ppiotr3k): write tests
// const MESSAGE_ID_COPY_DONE: u8 = b'c'; //TODO(ppiotr3k): write tests
// const MESSAGE_ID_COPY_FAIL: u8 = b'f'; //TODO(ppiotr3k): write tests
// const MESSAGE_ID_DESCRIBE: u8 = b'D'; //TODO(ppiotr3k): write tests
// const MESSAGE_ID_FUNCTION_CALL: u8 = b'F'; //TODO(ppiotr3k): write tests
// const MESSAGE_ID_GSSENC_REQUEST: u8 = b''; // ! no id //TODO(ppiotr3k): write tests
// const MESSAGE_ID_GSS_RESPONSE: u8 = b'p'; // ! shared id //TODO(ppiotr3k): write tests
// const MESSAGE_ID_PARSE: u8 = b'P'; //TODO(ppiotr3k): write tests
// const MESSAGE_ID_PASSWORD_MESSAGE: u8 = b'p'; // ! shared id //TODO(ppiotr3k): write tests

///TODO(ppiotr3k): write description
//TODO(ppiotr3k): investigate if `Clone` is avoidable; currently only used in tests
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    NotImplemented(Bytes),

    //#[cfg(test)] //TODO(ppiotr3k): fix enabling `Canary` only in tests
    Canary(u8),

    Bind {
        portal: Bytes,
        stmt_name: Bytes,
        parameters: Vec<BindParameter>,
        results_formats: Vec<u16>,
    },
    Execute {
        portal: Bytes,
        max_rows: u32,
    },
    Flush(),
    Query(Bytes),
    SASLInitialResponse {
        mecanism: Bytes,
        response: Bytes,
    },
    SASLResponse(Bytes),
    SSLRequest(),
    StartupMessage {
        frame_length: usize,
        parameters: Vec<Parameter>,
    },
    Sync(),
    Terminate(),

    //TODO(ppiotr3k): implement following messages
    CancelRequest(Bytes),
    Close(Bytes),
    CopyData(Bytes),
    CopyDone(Bytes),
    CopyFail(Bytes),
    Describe(Bytes),
    FunctionCall(Bytes),
    GSSENCRequest(Bytes),
    GSSResponse(Bytes),
    Parse(Bytes),
    PasswordMessage(Bytes),
}

impl PostgresMessage for Message {}
impl SQLMessage for Message {}

///TODO(ppiotr3k): write description
//TODO(ppiotr3k): internal fields encapsulation
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindParameter {
    pub format: u16,
    pub value: Bytes,
}

///TODO(ppiotr3k): write description
//TODO(ppiotr3k): internal fields encapsulation
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: Bytes,
    pub value: Bytes,
}

///TODO(ppiotr3k): write description
#[derive(Debug, Clone)]
enum DecodeState {
    Startup,
    Head,
    Message(usize),
}

///TODO(ppiotr3k): write description
#[derive(Debug, Clone)]
pub struct Codec {
    /// Read state management / optimization.
    state: DecodeState,
}

impl Codec {
    ///TODO(ppiotr3k): write function description
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: DecodeState::Startup,
        }
    }

    /// Transitions decoder from `Startup` to next state.
    pub fn startup_complete(&mut self) {
        self.state = DecodeState::Head;
    }

    ///TODO(ppiotr3k): write function description
    fn decode_header(&mut self, src: &mut BytesMut) -> io::Result<Option<usize>> {
        if src.len() < BYTES_MESSAGE_HEADER {
            // Incomplete header, await for more data.
            log::trace!(
                "not enough header data ({} bytes), awaiting more ({} bytes)",
                src.len(),
                BYTES_MESSAGE_HEADER,
            );
            return Ok(None);
        }

        // Peek into data with a `Cursor` to avoid advancing underlying buffer.
        let mut buf = io::Cursor::new(&mut *src);
        buf.advance(BYTES_MESSAGE_ID);

        // 'Message Length' field accounts for self, but not 'Message ID' field.
        // Note: `usize` prevents from 'Message Length' `i32` value overflow.
        let frame_length = (buf.get_u32() as usize) + BYTES_MESSAGE_ID;

        // Strict "less than", as null-payload messages exist in protocol.
        if frame_length < BYTES_MESSAGE_HEADER {
            log::trace!("invalid frame: {:?}", buf);
            let err = io::Error::new(
                io::ErrorKind::InvalidInput,
                "malformed packet - invalid message length",
            );
            log::error!("{}", err);
            return Err(err);
        }

        Ok(Some(frame_length))
    }

    ///TODO(ppiotr3k): write function description
    fn decode_message(&mut self, len: usize, src: &mut BytesMut) -> io::Result<Option<Message>> {
        if src.len() < len {
            // Incomplete message, await for more data.
            log::trace!(
                "not enough message data ({} bytes), awaiting more ({} bytes)",
                src.len(),
                len
            );
            return Ok(None);
        }

        // Full message, pop it out.
        let mut frame = src.split_to(len);
        //TODO(ppiotr3k): consider zero-cost `frame.freeze()` for lazy passing in `Pipe`

        // Frames have at least `BYTES_MESSAGE_HEADER` bytes at this point.
        let msg_id = frame.get_u8();
        log::trace!("incoming msg id: '{}' ({})", msg_id as char, msg_id);
        let msg_length = (frame.get_u32() as usize) - BYTES_MESSAGE_SIZE;
        log::trace!("incoming msg length: {}", msg_length);

        let msg = match msg_id {

            // Canary
            //#[cfg(test)] //TODO(ppiotr3k): fix enabling `Canary` only in tests
            b'B' /* 0x42 */ => {
                frame.advance(msg_length);
                Message::Canary(len as u8)
            },
            //#[cfg(test)] //TODO(ppiotr3k): fix enabling `Canary` only in tests
            b'!' /* 0x21 */ => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "expected canary error"));
            },

            // Frontend
            // MESSAGE_ID_BIND => {
            //     //TODO(ppiotr3k): implement this message
            // },
            MESSAGE_ID_EXECUTE => {
                let portal = get_cstr(&mut frame)?;
                let max_rows = get_u32(&mut frame, "malformed packet - invalid execute data")?;
                Message::Execute { portal, max_rows }
            },
            MESSAGE_ID_FLUSH => Message::Flush(),
            MESSAGE_ID_QUERY => {
                let query = get_cstr(&mut frame)?;
                Message::Query(query)
            },
            MESSAGE_ID_SASL => {
                // `SASLInitialResponse` holds a C-style null char terminated string,
                // while `SASlResponse` holds bytes with no 0 byte at all in them.
                // Therefore trying first to look for a `SASLInitialResponse`.
                //TODO(ppiotr3k): rethink, as `get_cstr` writes errors to logs
                // -> peeking at last frame byte and looking for a 0 maybe?
                if let Ok(mecanism) = get_cstr(&mut frame) {
                    const SASL_RESPONSE_SIZE_BYTES: usize = 4;
                    let response = get_bytes(
                        &mut frame,
                        SASL_RESPONSE_SIZE_BYTES,
                        "malformed packet - invalid SASL response data",
                    )?;

                    Message::SASLInitialResponse { mecanism, response }
                } else {
                    let response = frame.copy_to_bytes(frame.remaining());

                    // SASLResponse `response` field cannot be empty.
                    if response.is_empty() {
                        let err = std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "malformed packet - invalid SASL response data",
                        );
                        log::error!("{}", err);
                        return Err(err);
                    }

                    Message::SASLResponse(response)
                }
            },
            MESSAGE_ID_SYNC => Message::Sync(),
            MESSAGE_ID_TERMINATE => Message::Terminate(),
            _ => {
                let bytes = frame.copy_to_bytes(msg_length);
                Message::NotImplemented(bytes)
            },
        };

        // At this point, all data should have been consumed from `frame`.
        if !frame.is_empty() {
            log::trace!("invalid frame: {:?}", frame);
            let err = std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "malformed packet - invalid message length",
            );
            log::error!("{}", err);
            return Err(err);
        }

        log::debug!("decoded message frame: {:?}", msg);
        Ok(Some(msg))
    }

    ///TODO(ppiotr3k): write function description
    pub fn decode_startup_message(&mut self, src: &mut BytesMut) -> io::Result<Option<Message>> {
        if src.len() < BYTES_STARTUP_MESSAGE_HEADER {
            // Incomplete message, await for more data.
            log::trace!(
                "not enough header data ({} bytes), awaiting more ({} bytes)",
                src.len(),
                BYTES_STARTUP_MESSAGE_HEADER,
            );
            return Ok(None);
        }

        // Peek into data with a `Cursor` to avoid advancing underlying buffer.
        let mut buf = io::Cursor::new(&mut *src);

        // Note: `usize` prevents from 'Message Length' `i32` value overflow.
        let frame_length = buf.get_u32() as usize;
        if src.len() < frame_length {
            // Incomplete message, await for more data.
            log::trace!(
                "not enough message data ({} bytes), awaiting more ({} bytes)",
                src.len(),
                frame_length,
            );
            return Ok(None);
        }

        // Full message, pop it out.
        let mut frame = src.split_to(frame_length);
        log::trace!("decoded frame length: {}", frame_length);
        //TODO(ppiotr3k): consider zero-cost `frame.freeze()` for lazy passing in `Pipe`

        frame.advance(4); // `Message Length`

        let msg_id = frame.get_i32();
        log::trace!("msg id: {}", msg_id);
        let msg = match msg_id {
            MESSAGE_ID_STARTUP_MESSAGE => {
                let mut parameters = Vec::new();
                let mut user_param_exists = false;

                // At least one parameter and name/value pair terminator are expected.
                while frame.remaining() > 2 {
                    let parameter_name = get_cstr(&mut frame)?;

                    // Note: `user` is the sole required parameter, others are optional.
                    if parameter_name == "user" {
                        user_param_exists = true;
                    }

                    let parameter = Parameter {
                        name: parameter_name,
                        value: get_cstr(&mut frame)?,
                    };
                    log::trace!("decoded parameter: {:?}", parameter);
                    parameters.push(parameter);
                }

                // At this point, only name/value pair terminator should remain,
                // and a parameter named `user` should have been found.
                if frame.remaining() < 1 || !user_param_exists {
                    let err = std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "malformed packet - missing parameter fields",
                    );
                    log::error!("{}", err);
                    return Err(err);
                }
                frame.advance(1); // name/value pair terminator

                Message::StartupMessage {
                    frame_length,
                    parameters,
                }
            }
            MESSAGE_ID_SSL_REQUEST => Message::SSLRequest(),
            _ => {
                // If neither a recognized `StartupMessage` nor `SSLRequest`,
                // consider as `StartupMessage` with unsupported protocol version.
                let err = std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "malformed packet - invalid protocol version",
                );
                log::error!("{}", err);
                return Err(err);
            }
        };
        log::debug!("decoded message frame: {:?}", msg);
        Ok(Some(msg))
    }

    ///TODO(ppiotr3k): write function description
    //TODO(ppiotr3k): get size from Message struct
    // -> pre-requisite: enum variants are considered as types in Rust
    fn encode_header(&mut self, msg_id: u8, msg_size: usize, dst: &mut BytesMut) {
        dst.reserve(BYTES_MESSAGE_HEADER + msg_size);
        dst.put_u8(msg_id);
        dst.put_u32((BYTES_MESSAGE_SIZE + msg_size) as u32);
    }
}

impl Decoder for Codec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        log::trace!("decoder state: {:?}", self.state);
        let msg_length = match self.state {
            // During startup sequence, frontend can send an `SSLRequest` message rather
            // than a `StartupMessage`. `Startup` state handles this initial edge case.
            // https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.12
            DecodeState::Startup => match self.decode_startup_message(src)? {
                None => return Ok(None),
                Some(Message::SSLRequest()) => return Ok(Some(Message::SSLRequest())),
                Some(Message::StartupMessage {
                    frame_length,
                    parameters,
                }) => {
                    self.startup_complete();
                    return Ok(Some(Message::StartupMessage {
                        frame_length,
                        parameters,
                    }));
                }
                Some(other) => {
                    let err = io::Error::new(
                        io::ErrorKind::InvalidData,
                        //TODO(ppiotr3k): rewrite without debug symbols
                        format!("unexpected message during startup: {:?}", other),
                    );
                    log::error!("{}", err);
                    return Err(err);
                }
            },

            DecodeState::Head => match self.decode_header(src)? {
                // Incomplete header, await for more data.
                None => return Ok(None),
                // Header available, try getting full message.
                Some(length) => {
                    self.state = DecodeState::Message(length);

                    // Ensure enough space is available to read incoming payload.
                    // Note: acceptable over-allocation by content of `BYTES_MESSAGE_SIZE`.
                    src.reserve(length);
                    log::trace!("stream buffer capacity: {} bytes", src.capacity());

                    length
                }
            },

            DecodeState::Message(length) => length,
        };
        log::trace!("decoded frame length: {} bytes", msg_length);

        match self.decode_message(msg_length, src)? {
            // Incomplete message, await for more data.
            None => Ok(None),
            // Full message, pop it out, move on to parsing a new one.
            Some(msg) => {
                self.state = DecodeState::Head;

                // Ensure enough space is available to read next header.
                src.reserve(BYTES_MESSAGE_HEADER);
                log::trace!("stream buffer capacity: {} bytes", src.capacity());

                Ok(Some(msg))
            }
        }
    }
}

impl Encoder<Message> for Codec {
    type Error = io::Error;

    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), io::Error> {
        //TODO(ppiotr3k): rationalize capacity reservation with `dst.reserve(msg.len())`
        // -> pre-requisite: enum variants are considered as types in Rust
        match msg {
            Message::Execute { portal, max_rows } => {
                self.encode_header(MESSAGE_ID_EXECUTE, portal.len() + 1 + 4, dst);
                put_cstr(&portal, dst);
                dst.put_i32(max_rows as i32);
            }
            Message::Flush() => {
                self.encode_header(MESSAGE_ID_FLUSH, 0, dst);
            }
            Message::Query(query) => {
                self.encode_header(MESSAGE_ID_QUERY, query.len() + 1, dst);
                put_cstr(&query, dst);
            }
            Message::SASLInitialResponse { mecanism, response } => {
                self.encode_header(
                    MESSAGE_ID_SASL,
                    mecanism.len() + 1 + 4 + response.len(),
                    dst,
                );
                put_cstr(&mecanism, dst);
                put_bytes(&response, dst);
            }
            Message::SASLResponse(response) => {
                self.encode_header(MESSAGE_ID_SASL, response.len(), dst);
                dst.put(response);
            }
            Message::StartupMessage {
                frame_length,
                parameters,
            } => {
                dst.reserve(frame_length);
                dst.put_i32(frame_length as i32);
                dst.put_i32(196608);
                for parameter in &parameters {
                    put_cstr(&parameter.name, dst);
                    put_cstr(&parameter.value, dst);
                }
                dst.put_u8(0); // name/value pair terminator
            }
            Message::SSLRequest() => {
                dst.reserve(8);
                dst.put_i32(8);
                dst.put_i32(80877103);
            }
            Message::Sync() => {
                self.encode_header(MESSAGE_ID_SYNC, 0, dst);
            }
            Message::Terminate() => {
                self.encode_header(MESSAGE_ID_TERMINATE, 0, dst);
            }
            other => {
                unimplemented!("not implemented: {:?}", other)
            }
        }

        // Message has been written to `Sink`, nothing left to do.
        // Note: if bytes remain in frame, encoding tests need a review.
        Ok(())
    }
}

impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod decode_tests {

    use bytes::{Bytes, BytesMut};
    use test_log::test;

    use super::{Codec, Message, Parameter};

    /// Helper function to ease writing decoding tests for startup sequence.
    fn assert_decode_startup_message(data: &[u8], expected: &[Message], remaining: usize) {
        let buf = &mut BytesMut::from(data);
        let mut decoded = Vec::new();

        let mut codec = Codec::new();
        while let Ok(Some(msg)) = codec.decode_startup_message(buf) {
            decoded.push(msg);
        }

        assert_eq!(remaining, buf.len(), "remaining bytes in read buffer");
        assert_eq!(expected.len(), decoded.len(), "decoded messages");
        assert_eq!(expected, decoded, "decoded messages");
    }

    #[test]
    #[rustfmt::skip]
    fn valid_startup_message() {
        let data = [
            0, 0, 0, 78,                                                                  // total length: 78
            0, 3, 0, 0,                                                                   // protocol version: 3.0
            117, 115, 101, 114, 0,                                                        // cstr: "user\0"
            114, 111, 111, 116, 0,                                                        // cstr: "root\0"
            100, 97, 116, 97, 98, 97, 115, 101, 0,                                        // cstr: "database\0"
            116, 101, 115, 116, 100, 98, 0,                                               // cstr: "testdb\0"
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, // cstr: "application_name\0"
            112, 115, 113, 108, 0,                                                        // cstr: "psql\0"
            99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0,    // cstr: "client_encoding\0"
            85, 84, 70, 56, 0,                                                            // cstr: "UTF8\0"
            0,                                                                            // name/value pair terminator
        ];

        let expected = vec![
            Message::StartupMessage {
                frame_length: 78,
                parameters: vec![
                    Parameter {
                        name: Bytes::from_static(b"user"),
                        value: Bytes::from_static(b"root"),
                    },
                    Parameter {
                        name: Bytes::from_static(b"database"),
                        value: Bytes::from_static(b"testdb"),
                    },
                    Parameter {
                        name: Bytes::from_static(b"application_name"),
                        value: Bytes::from_static(b"psql"),
                    },
                    Parameter {
                        name: Bytes::from_static(b"client_encoding"),
                        value: Bytes::from_static(b"UTF8"),
                    },
                ]},
        ];
        let remaining = 0;

        assert_decode_startup_message(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_startup_message_wrong_protocol_version() {
        let data = [
            0, 0, 0, 78,                                                                  // total length: 78
            0, 2, 0, 0,                                                                   // wrong protocol version: 2.0
            117, 115, 101, 114, 0,                                                        // cstr: "user\0"
            114, 111, 111, 116, 0,                                                        // cstr: "root\0"
            100, 97, 116, 97, 98, 97, 115, 101, 0,                                        // cstr: "database\0"
            116, 101, 115, 116, 100, 98, 0,                                               // cstr: "testdb\0"
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, // cstr: "application_name\0"
            112, 115, 113, 108, 0,                                                        // cstr: "psql\0"
            99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0,    // cstr: "client_encoding\0"
            85, 84, 70, 56, 0,                                                            // cstr: "UTF8\0"
            0,                                                                            // name/value pair terminator
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode_startup_message(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_startup_message_missing_required_user() {
        let data = [
            0, 0, 0, 68,                                                                  // total length: 68
            0, 3, 0, 0,                                                                   // protocol version: 3.0
            100, 97, 116, 97, 98, 97, 115, 101, 0,                                        // cstr: "database\0"
            116, 101, 115, 116, 100, 98, 0,                                               // cstr: "testdb\0"
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, // cstr: "application_name\0"
            112, 115, 113, 108, 0,                                                        // cstr: "psql\0"
            99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0,    // cstr: "client_encoding\0"
            85, 84, 70, 56, 0,                                                            // cstr: "UTF8\0"
            0,                                                                            // name/value pair terminator
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode_startup_message(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_startup_message_empty_parameters_list() {
        let data = [
            0, 0, 0, 9, // total length: 9
            0, 3, 0, 0, // protocol version: 3.0
            0,          // name/value pair terminator
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode_startup_message(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_startup_message_missing_parameters_data() {
        let data = [
            0, 0, 0, 8, // total length: 8
            0, 3, 0, 0, // protocol version: 3.0
                        // missing parameters data
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode_startup_message(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_startup_message_missing_parameters_list_terminator() {
        let data = [
            0, 0, 0, 77,                                                                  // total length: 77
            0, 3, 0, 0,                                                                   // protocol version: 3.0
            117, 115, 101, 114, 0,                                                        // cstr: "user\0"
            114, 111, 111, 116, 0,                                                        // cstr: "root\0"
            100, 97, 116, 97, 98, 97, 115, 101, 0,                                        // cstr: "database\0"
            116, 101, 115, 116, 100, 98, 0,                                               // cstr: "testdb\0"
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, // cstr: "application_name\0"
            112, 115, 113, 108, 0,                                                        // cstr: "psql\0"
            99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0,    // cstr: "client_encoding\0"
            85, 84, 70, 56, 0,                                                            // cstr: "UTF8\0"
                                                                                          // missing name/value pair terminator
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode_startup_message(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_startup_message_missing_parameter_field() {
        let data = [
            0, 0, 0, 28,                           // total length: 28
            0, 3, 0, 0,                            // protocol version: 3.0
            117, 115, 101, 114, 0,                 // cstr: "user\0"
            114, 111, 111, 116, 0,                 // cstr: "root\0"
            100, 97, 116, 97, 98, 97, 115, 101, 0, // cstr: "database\0"
            0,                                     // missing value field || missing name/value pair terminator
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode_startup_message(&data[..], &expected, remaining);
    }
}
