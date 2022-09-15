// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

//! [`Decoder`]/[`Encoder`] traits implementations
//! for PostgreSQL backend Messages.
//!
//! [`Decoder`]: https://docs.rs/tokio-util/*/tokio_util/codec/trait.Decoder.html
//! [`Encoder`]: https://docs.rs/tokio-util/*/tokio_util/codec/trait.Encoder.html

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

use super::{PostgresMessage, SQLMessage};
use crate::codec::constants::*;
use crate::codec::utils::*;

const MESSAGE_ID_AUTHENTICATION: u8 = b'R';
const MESSAGE_ID_BACKEND_KEY_DATA: u8 = b'K';
const MESSAGE_ID_COMMAND_COMPLETE: u8 = b'C';
const MESSAGE_ID_DATA_ROW: u8 = b'D';
const MESSAGE_ID_EMPTY_QUERY_RESPONSE: u8 = b'I';
const MESSAGE_ID_ERROR_RESPONSE: u8 = b'E'; //TODO(ppiotr3k): write tests
const MESSAGE_ID_PARAMETER_STATUS: u8 = b'S';
const MESSAGE_ID_READY_FOR_QUERY: u8 = b'Z';
const MESSAGE_ID_ROW_DESCRIPTION: u8 = b'T';

//TODO(ppiotr3k): implement following messages
// const MESSAGE_ID_AUTHENTICATION_KERBEROS_V5: u8 = b'R'; // 2
// const MESSAGE_ID_AUTHENTICATION_CLEARTEXT_PASSWORD: u8 = b'R'; // 3
// const MESSAGE_ID_AUTHENTICATION_MD5_PASSWORD: u8 = b'R'; // 5
// const MESSAGE_ID_AUTHENTICATION_SCM_CREDENTIAL: u8 = b'R'; // 6
// const MESSAGE_ID_AUTHENTICATION_GSS: u8 = b'R'; // 7
// const MESSAGE_ID_AUTHENTICATION_GSS_CONTINUE: u8 = b'R'; // 8
// const MESSAGE_ID_AUTHENTICATION_SSPI: u8 = b'R'; // 9
// const MESSAGE_ID_BIND_COMPLETE: u8 = b'2';
// const MESSAGE_ID_CLOSE_COMPLETE: u8 = b'3';
// const MESSAGE_ID_COPY_DATA: u8 = b'd';
// const MESSAGE_ID_COPY_DONE: u8 = b'c';
// const MESSAGE_ID_COPY_IN_RESPONSE: u8 = b'G';
// const MESSAGE_ID_COPY_OUT_RESPONSE: u8 = b'H';
// const MESSAGE_ID_COPY_BOTH_RESPONSE: u8 = b'W';
// const MESSAGE_ID_FUNCTION_CALL_RESPONSE: u8 = b'V';
// const MESSAGE_ID_NEGOTIATE_PROTOCOL_VERSION: u8 = b'v';
// const MESSAGE_ID_NO_DATA: u8 = b'n';
// const MESSAGE_ID_NOTICE_RESPONSE: u8 = b'N';
// const MESSAGE_ID_NOTIFICATION_RESPONSE: u8 = b'A';
// const MESSAGE_ID_PARAMETER_DESCRIPTION: u8 = b'B';
// const MESSAGE_ID_PARSE_COMPLETE: u8 = b'1';
// const MESSAGE_ID_PORTAL_SUSPENDED: u8 = b's';

///TODO(ppiotr3k): write description
//TODO(ppiotr3k): investigate if `Clone` is avoidable; currently only used in tests
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    NotImplemented(Bytes),

    //#[cfg(test)] //TODO(ppiotr3k): fix enabling `Canary` only in tests
    Canary(u8),

    AuthenticationOk(),
    AuthenticationSASL(Bytes),
    AuthenticationSASLContinue(Bytes),
    AuthenticationSASLFinal(Bytes),
    CommandComplete(Bytes),
    BackendKeyData { process: u32, secret_key: u32 },
    DataRow(Vec<Bytes>),
    EmptyQueryResponse(),
    ErrorResponse(Bytes),
    ParameterStatus { parameter: Bytes, value: Bytes },
    ReadyForQuery(u8),
    RowDescription(Vec<RowDescription>),

    //TODO(ppiotr3k): implement following messages
    AuthenticationKerberosV5(Bytes),
    AuthenticationCleartextPassword(Bytes),
    AuthenticationMD5Password(Bytes),
    AuthenticationSCMCredential(Bytes),
    AuthenticationGSS(Bytes),
    AuthenticationGSSContinue(Bytes),
    AuthenticationSSPI(Bytes),
    BindComplete(Bytes),
    CloseComplete(Bytes),
    CopyData(Bytes),
    CopyDone(Bytes),
    CopyInResponse(Bytes),
    CopyOutResponse(Bytes),
    CopyBothResponse(Bytes),
    FunctionCallResponse(Bytes),
    NegotiateProtocolVersion(Bytes),
    NoData(),
    NoticeResponse(Bytes),
    NotificationResponse(Bytes),
    ParameterDescription(Bytes),
    ParseComplete(),
    PortalSuspended(),
}

///TODO(ppiotr3k): write description
//TODO(ppiotr3k): investigate if `Clone` is avoidable; currently only used in tests
//TODO(ppiotr3k): internal fields encapsulation
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowDescription {
    pub name: Bytes,
    pub table_oid: u32,
    pub column_attr: u16,
    pub data_type_oid: u32,
    pub data_type_size: i16,
    pub type_modifier: i32,
    pub format: u16,
}

///TODO(ppiotr3k): write description
#[derive(Debug, Clone)]
enum DecodeState {
    Head,
    Message(usize),
}

///TODO(ppiotr3k): write description
#[derive(Debug, Clone)]
pub struct Codec {
    /// Read state management / optimization
    state: DecodeState,
}

impl Codec {
    ///TODO(ppiotr3k): write function description
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: DecodeState::Head,
        }
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

        let mut buf = io::Cursor::new(&mut *src);
        buf.advance(BYTES_MESSAGE_ID);

        // 'Message Length' field accounts for self, but not 'Message ID' field.
        // Note: `usize` prevents from 'Message Length' `i32` value overflow.
        let frame_length = (buf.get_u32() as usize) + BYTES_MESSAGE_ID;

        // Strict "less than", as null-payload messages exist in protocol.
        if frame_length < BYTES_MESSAGE_HEADER {
            log::trace!("invalid frame: {:?}", buf);
            let err = std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
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

            // Backend
            MESSAGE_ID_AUTHENTICATION => {
                let authn_case = get_u32(&mut frame, "malformed packet - invalid authentication data")?;
                match authn_case {
                     0 /* AuthenticationOk */=> Message::AuthenticationOk(),
                    10 /* AuthenticationSASL */ => {
                        let data = get_cstr(&mut frame)?;

                        // A zero byte is required as terminator after the last authn mechanism.
                        //TODO(ppiotr3k): write a test where it is a different value than zero
                        if frame.is_empty() {
                            let err = std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "malformed packet - invalid SASL mecanism data",
                            );
                            log::error!("{}", err);
                            return Err(err);
                        }
                        frame.advance(1); // zero byte list terminator

                        Message::AuthenticationSASL(data)
                    },
                    11 /* AuthenticationSASLContinue */ => {
                        let response = frame.copy_to_bytes(frame.remaining());

                        // AuthenticationSASLContinue `response` cannot be empty.
                        if response.is_empty() {
                            let err = std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "malformed packet - invalid SASL response data",
                            );
                            log::error!("{}", err);
                            return Err(err);
                        }

                        Message::AuthenticationSASLContinue(response)
                    },
                    12 /* AuthenticationSASLFinal */ => {
                        let response = frame.copy_to_bytes(frame.remaining());

                        // AuthenticationSASLFinal `response` cannot be empty.
                        if response.is_empty() {
                            let err = std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "malformed packet - invalid SASL response data",
                            );
                            log::error!("{}", err);
                            return Err(err);
                        }

                        Message::AuthenticationSASLFinal(response)
                    },
                    _ => {
                        let err = std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "malformed packet - invalid SASL identifier",
                        );
                        log::error!("{}", err);
                        return Err(err);
                    }
                }
            },
            MESSAGE_ID_BACKEND_KEY_DATA => {
                let process = get_u32(&mut frame, "malformed packet - invalid key data")?;
                let secret_key = get_u32(&mut frame, "malformed packet - invalid key data")?;
                Message::BackendKeyData { process, secret_key }
            },
            MESSAGE_ID_COMMAND_COMPLETE => {
                let command = get_cstr(&mut frame)?;
                Message::CommandComplete(command)
            },
            MESSAGE_ID_DATA_ROW => {
                let fields = self.get_data_row_fields(&mut frame)?;
                Message::DataRow(fields)
            },
            MESSAGE_ID_ERROR_RESPONSE => {
                //TODO(ppiotr3k): identify if parsing those fields is of interest
                let unparsed_fields = frame.copy_to_bytes(msg_length);
                Message::ErrorResponse(unparsed_fields)
            },
            MESSAGE_ID_EMPTY_QUERY_RESPONSE => Message::EmptyQueryResponse(),
            MESSAGE_ID_PARAMETER_STATUS => {
                let parameter = get_cstr(&mut frame)?;
                let value = get_cstr(&mut frame)?;
                Message::ParameterStatus { parameter, value }
            },
            MESSAGE_ID_READY_FOR_QUERY => {
                let status = get_u8(&mut frame, "malformed packet - missing status indicator")?;
                match status {
                    b'I' | b'T'| b'E' => Message::ReadyForQuery(status),
                    _ => {
                        let err = std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "malformed packet - invalid status indicator",
                        );
                        log::error!("{}", err);
                        return Err(err);
                    },
                }
            },
            MESSAGE_ID_ROW_DESCRIPTION => {
                let descriptions = self.get_row_descriptions(&mut frame)?;
                Message::RowDescription(descriptions)
            },
            _ => {
                let bytes = frame.copy_to_bytes(msg_length);
                unimplemented!("msg_id: {} ({:?})", msg_id, bytes);
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
    fn get_row_descriptions(&mut self, buf: &mut BytesMut) -> io::Result<Vec<RowDescription>> {
        let mut columns = get_u16(buf, "malformed packet - invalid data size")?;
        log::trace!("decoded number of description columns: {}", columns);

        let mut decoded = Vec::new();

        const BYTES_ROW_DESCRIPTION_COMMON_LENGTH: usize = 18;
        while columns > 0 {
            let column_name = get_cstr(buf)?;

            if buf.remaining() < BYTES_ROW_DESCRIPTION_COMMON_LENGTH {
                let err = std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "malformed packet - invalid row description structure",
                );
                log::error!("{}", err);
                return Err(err);
            }

            let description = RowDescription {
                name: column_name,
                table_oid: get_u32(buf, "malformed packet - invalid data size")?,
                column_attr: get_u16(buf, "malformed packet - invalid data size")?,
                data_type_oid: get_u32(buf, "malformed packet - invalid data size")?,
                data_type_size: get_i16(buf, "malformed packet - invalid data size")?,
                type_modifier: get_i32(buf, "malformed packet - invalid data size")?,
                format: get_u16(buf, "malformed packet - invalid data size")?,
            };

            log::trace!("decoded row description: {:?}", description);
            decoded.push(description);
            columns -= 1;
        }

        Ok(decoded)
    }

    ///TODO(ppiotr3k): write function description
    fn get_data_row_fields(&mut self, buf: &mut BytesMut) -> io::Result<Vec<Bytes>> {
        let mut fields = buf.get_u16();
        log::trace!("decoded number of row fields: {}", fields);

        let mut decoded = Vec::new();

        const BYTES_DATA_ROW_FIELD_LENGTH: usize = 4;
        while fields > 0 {
            let value = get_bytes(
                buf,
                BYTES_DATA_ROW_FIELD_LENGTH,
                "malformed packet - invalid field size",
            )?;

            log::trace!("decoded field: {:?}", value);
            decoded.push(value);
            fields -= 1;
        }

        Ok(decoded)
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

impl PostgresMessage for Message {}
impl SQLMessage for Message {}

impl Decoder for Codec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        let msg_length = match self.state {
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
            Message::AuthenticationOk() => {
                self.encode_header(MESSAGE_ID_AUTHENTICATION, 4, dst);
                dst.put_i32(0);
            }
            Message::AuthenticationSASL(data) => {
                self.encode_header(MESSAGE_ID_AUTHENTICATION, 4 + data.len() + 1 + 1, dst);
                dst.put_i32(10);
                put_cstr(&data, dst);
                dst.put_u8(0); // zero byte list terminator
            }
            Message::AuthenticationSASLContinue(response) => {
                self.encode_header(MESSAGE_ID_AUTHENTICATION, 4 + response.len(), dst);
                dst.put_i32(11);
                dst.put(response);
            }
            Message::AuthenticationSASLFinal(response) => {
                self.encode_header(MESSAGE_ID_AUTHENTICATION, 4 + response.len(), dst);
                dst.put_i32(12);
                dst.put(response);
            }
            Message::BackendKeyData {
                process,
                secret_key,
            } => {
                self.encode_header(MESSAGE_ID_BACKEND_KEY_DATA, 4 + 4, dst);
                dst.put_i32(process as i32);
                dst.put_i32(secret_key as i32);
            }
            Message::CommandComplete(command) => {
                self.encode_header(MESSAGE_ID_COMMAND_COMPLETE, command.len() + 1, dst);
                put_cstr(&command, dst);
            }
            Message::DataRow(fields) => {
                let mut msg_size = 2;
                for field in fields.iter() {
                    msg_size += field.len() + 4;
                }

                self.encode_header(MESSAGE_ID_DATA_ROW, msg_size, dst);
                dst.put_u16(fields.len() as u16);

                for field in fields.iter() {
                    put_bytes(field, dst)
                }
            }
            Message::EmptyQueryResponse() => {
                self.encode_header(MESSAGE_ID_EMPTY_QUERY_RESPONSE, 0, dst);
            }
            Message::ErrorResponse(unparsed_fields) => {
                self.encode_header(MESSAGE_ID_ERROR_RESPONSE, unparsed_fields.len(), dst);
                dst.put(unparsed_fields);
            }
            Message::ParameterStatus { parameter, value } => {
                self.encode_header(
                    MESSAGE_ID_PARAMETER_STATUS,
                    parameter.len() + 1 + value.len() + 1,
                    dst,
                );
                put_cstr(&parameter, dst);
                put_cstr(&value, dst);
            }
            Message::ReadyForQuery(status) => {
                self.encode_header(MESSAGE_ID_READY_FOR_QUERY, 1, dst);
                dst.put_u8(status);
            }
            Message::RowDescription(descriptions) => {
                let mut msg_size = 2;
                for column in descriptions.iter() {
                    msg_size += column.name.len() + 1 + 4 + 2 + 4 + 2 + 4 + 2;
                }

                self.encode_header(MESSAGE_ID_ROW_DESCRIPTION, msg_size, dst);
                dst.put_u16(descriptions.len() as u16);

                for column in descriptions.iter() {
                    put_cstr(&column.name, dst);
                    dst.put_u32(column.table_oid);
                    dst.put_u16(column.column_attr);
                    dst.put_u32(column.data_type_oid);
                    dst.put_i16(column.data_type_size);
                    dst.put_i32(column.type_modifier);
                    dst.put_u16(column.format);
                }
            }
            other => {
                unimplemented!("msg: {:?}", other)
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
