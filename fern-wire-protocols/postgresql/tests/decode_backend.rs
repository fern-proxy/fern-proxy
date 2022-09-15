// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

mod common;

mod decode_backend {

    use bytes::{Bytes, BytesMut};
    use test_log::test;
    use tokio_util::codec::Decoder;

    use fern_protocol_postgresql::codec::backend::{Codec, Message, RowDescription};

    /// Helper function to ease writing decoding tests.
    fn assert_decode(data: &[u8], expected: &[Message], remaining: usize) {
        let buf = &mut BytesMut::from(data);
        let mut decoded = Vec::new();

        let mut codec = Codec::new();

        while let Ok(Some(msg)) = codec.decode(buf) {
            decoded.push(msg);
        }

        assert_eq!(remaining, buf.len(), "remaining bytes in read buffer");
        assert_eq!(expected.len(), decoded.len(), "decoded messages");
        assert_eq!(expected, decoded, "decoded messages");
    }

    #[test]
    #[rustfmt::skip]
    fn valid_decode_canary_remaining_data() {
        let data = [
            0x42u8,                                                                // known header byte
            0x0, 0x0, 0x0, 0x10,                                                   // length: 16
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x0, // payload: "hello world\0"
            0x42u8,                                                                // known header byte
            0x0, 0x0, 0x0, 0xe,                                                    // length: 14
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x61, 0x72, 0x0,             // payload: "hello war\0"
            0x42u8, 0x0, 0x0,                                                      // extra valid bytes
        ];

        let expected = vec![
            Message::Canary(17),
            Message::Canary(15),
        ];
        let remaining = 3;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_decode_canary_null_payload_message() {
        let data = [
            0x42u8,             // known header byte
            0x0, 0x0, 0x0, 0x4, // length: 4
        ];

        let expected = vec![
            Message::Canary(5),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_decode_canary_null_size_message() {
        let data = [
            0x42u8,             // known header byte
            0x0, 0x0, 0x0, 0x0, // length: 0
        ];

        let expected = vec![];
        let remaining = 5;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_decode_canary_truncated_header() {
        let data = [
            0x42u8,        // known header byte
            0x0, 0x0, 0x0, // truncated length
        ];

        let expected = vec![];
        let remaining = 4;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_decode_canary_unknown_message_id() {
        let data = [
            0x21u8,                                                                // invalid header byte
            0x0, 0x0, 0x0, 0x10,                                                   // length: 12
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x0, // payload: "hello world\0"
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_ok() {
        let data = [
            82,         // msg id: 'R' - ! shared id
            0, 0, 0, 8, // payload length: 8
            0, 0, 0, 0, // const AuthN: success
        ];

        let expected = vec![
            Message::AuthenticationOk(),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_authentication_ok_missing_const() {
        let data = [
            82,         // msg id: 'R' - ! shared id
            0, 0, 0, 0, // payload length: 0
                        // missing const: AuthnN: success
        ];

        let expected = vec![];
        let remaining = 5;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_authentication_malformed_authn_const() {
        let data = [
            82,         // msg id: 'R' - ! shared id
            0, 0, 0, 7, // payload length: 7
            0, 0, 0,    // malformed const: AuthnN
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_authentication_unknown_authn_const() {
        let data = [
            82,          // msg id: 'R' - ! shared id
            0, 0, 0, 8,  // payload length: 8
            0, 0, 0, 42, // unknown const: AuthnN
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_sasl() {
        let data = [
            82,                                                    // msg id: 'R' - ! shared id
            0, 0, 0, 23,                                           // payload length: 23
            0, 0, 0, 10,                                           // const authn: SASL authn
            83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0, // cstr: "SCRAM-SHA-256\0"
            0,                                                     // list terminator
        ];

        let expected = vec![
            Message::AuthenticationSASL(
                Bytes::from_static(b"SCRAM-SHA-256"),
            ),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_authentication_sasl_truncated_payload() {
        let data = [
            82,         // msg id: 'R' - ! shared id
            0, 0, 0, 7, // payload length: 9
            0, 0, 0,    // truncated const authn
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_authentication_sasl_missing_list_terminator() {
        let data = [
            82,                                                    // msg id: 'R' - ! shared id
            0, 0, 0, 22,                                           // payload length: 22
            0, 0, 0, 10,                                           // const authn: SASL authn
            83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0, // cstr: "SCRAM-SHA-256\0"
                                                                   // missing list terminator
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_sasl_continue() {
        let data = [
            82,                                                                     // msg id: 'R' - ! shared id
            0, 0, 0, 92,                                                            // payload length: 92
            0, 0, 0, 11,                                                            // const authn: SASL continue
            114, 61, 79, 103, 111, 110, 89, 82, 110, 108, 48, 52, 97, 100, 103, 66, // SASL response
            51, 54, 83, 112, 76, 113, 111, 85, 52, 117, 83, 97, 52, 73, 110, 52,    // SASL response - cont'd
            115, 81, 122, 105, 90, 86, 106, 87, 50, 97, 112, 122, 66, 48, 48, 108,  // SASL response - cont'd
            111, 79, 44, 115, 61, 53, 120, 99, 76, 110, 48, 112, 49, 70, 90, 89,    // SASL response - cont'd
            52, 119, 106, 114, 79, 50, 115, 73, 49, 55, 119, 61, 61, 44, 105, 61,   // SASL response - cont'd
            52, 48, 57, 54,                                                         // SASL response - cont'd
        ];

        let expected = vec![
            Message::AuthenticationSASLContinue(
                Bytes::from_static(b"r=OgonYRnl04adgB36SpLqoU4uSa4In4sQziZVjW2apzB00loO,s=5xcLn0p1FZY4wjrO2sI17w==,i=4096"),
            ),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_authentication_sasl_continue_missing_response() {
        let data = [
            82,          // msg id: 'R' - ! shared id
            0, 0, 0, 8,  // payload length: 92
            0, 0, 0, 11, // const authn: SASL continue
                         // missing SASL response
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_sasl_final() {
        let data = [
            82,                                                                   // msg id: 'R' - ! shared id
            0, 0, 0, 54,                                                          // payload length: 54
            0, 0, 0, 12,                                                          // const authn: SASL final
            118, 61, 107, 87, 86, 99, 43, 117, 77, 56, 105, 99,                   // SASL response
            65, 48, 109, 106, 66, 106, 73, 86, 103, 48, 55, 113, 98, 56, 78, 108, // SASL response - cont'd
            114, 77, 82, 112, 75, 82, 72, 87, 114, 70, 98, 99, 100, 81, 74, 111,  // SASL response - cont'd
            119, 61,                                                              // SASL response - cont'd
        ];

        let expected = vec![
            Message::AuthenticationSASLFinal(
                Bytes::from_static(b"v=kWVc+uM8icA0mjBjIVg07qb8NlrMRpKRHWrFbcdQJow="),
            ),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_authentication_sasl_final_missing_response() {
        let data = [
            82,          // msg id: 'R' - ! shared id
            0, 0, 0, 8,  // payload length: 8
            0, 0, 0, 12, // const authn: SASL final
                         // missing SASL response
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_backend_key_data() {
        let data = [
            75,               // msg id: 'K'
            0, 0, 0, 12,      // payload lenght: 12
            0, 0, 0, 80,      // backend process id
            238, 248, 81, 43, // cancellation secret key
        ];

        let expected = vec![
            Message::BackendKeyData {
                process: 80,
                secret_key: 4009251115,
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_backend_key_data_null_size() {
        let data = [
            75,         // msg id: 'K'
            0, 0, 0, 4, // payload length: 4
                        // missing data
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_backend_key_data_missing_field() {
        let data = [
            75,          // msg id: 'K'
            0, 0, 0, 8,  // payload lenght: 12
            0, 0, 0, 80, // backend process id
                         // missing secret key
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_command_complete() {
        let data = [
            67,                                // msg id: 'C'
            0, 0, 0, 13,                       // payload length: 13
            83, 69, 76, 69, 67, 84, 32, 49, 0, // cstr: "SELECT 1\0"
        ];

        let expected = vec![
            Message::CommandComplete(Bytes::from_static(b"SELECT 1")),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_command_complete_null_size() {
        let data = [
            67, // msg id: 'C'
            0, 0, 0, 0, // payload length: 0
        ];

        let expected = vec![];
        let remaining = 5;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_simple() {
        let data = [
            68,          // msg id: 'D'
            0, 0, 0, 11, // payload length: 11
            0, 1,        // number of columns: 1

            0, 0, 0, 1,  // c1: field length: 1
            49,          // c1: field - schema specific (here: "1")
        ];

        let expected = vec![
            Message::DataRow(vec![
                Bytes::from_static(b"1"),
            ]),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_zero_columns() {
        let data = [
            68,         // msg id: 'D'
            0, 0, 0, 6, // payload length: 11
            0, 0,       // number of columns: 0
        ];

        let expected = vec![
            Message::DataRow(vec![]),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_data_row_missing_column() {
        let data = [
            68,          // msg id: 'D'
            0, 0, 0, 11, // payload length: 11
            0, 2,        // number of columns: 2

            0, 0, 0, 1,  // c1: col value length: 1
            49,          // c1: col value - schema specific (here: "1")

                         // missing c2
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_data_row_truncated_field() {
        let data = [
            68,          // msg id: 'D'
            0, 0, 0, 11, // payload length: 11
            0, 1,        // number of columns: 1

            0, 0, 0, 2,  // c1: col field length: 2
            49,          // truncated c1: col field
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_data_row_wrong_payload_length() {
        let data = [
            68,          // msg id: 'D'
            0, 0, 0, 12, // payload length: 12
            0, 1,        // number of columns: 1

            0, 0, 0, 1,  // c1: col value length: 2
            49,          // c1: col value - schema specific (here: "1")
            42,          // extra invalid byte
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_complex_null_columns_ending() {
        let data = [
            68,                                                      // msd id: 'D'
            0, 0, 0, 87,                                             // payload length: 87
            0, 8,                                                    // number of columns: 8

            0, 0, 0, 10,                                             // c1: field length: 10
            112, 103, 95, 99, 97, 116, 97, 108, 111, 103,            // c1: field - schema specific ("pg_catalog")

            0, 0, 0, 12,                                             // c2: field length: 12
            112, 103, 95, 97, 103, 103, 114, 101, 103, 97, 116, 101, // c2: field - schema specific ("pg_aggregate")

            0, 0, 0, 5,                                              // c3: field length: 5
            116, 97, 98, 108, 101,                                   // c3: field - schema specific ("table")

            0, 0, 0, 4,                                              // c4: field length: 4
            114, 111, 111, 116,                                      // c4: field - schema specific ("root")

            0, 0, 0, 9,                                              // c5: field length: 9
            112, 101, 114, 109, 97, 110, 101, 110, 116,              // c5: field - schema specific ("permanent")

            0, 0, 0, 4,                                              // c6: field length: 4
            104, 101, 97, 112,                                       // c6: field - schema specific ("heap")

            0, 0, 0, 5,                                              // c7: field length: 10
            53, 54, 32, 107, 66,                                     // c7: field - schema specific ("56 kB")

            255, 255, 255, 255,                                      // c8: NULL field, no value bytes
        ];

        let expected = vec![
            Message::DataRow(vec![
                Bytes::from_static(b"pg_catalog"),
                Bytes::from_static(b"pg_aggregate"),
                Bytes::from_static(b"table"),
                Bytes::from_static(b"root"),
                Bytes::from_static(b"permanent"),
                Bytes::from_static(b"heap"),
                Bytes::from_static(b"56 kB"),
                Bytes::from_static(b""),
            ]),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_complex_null_columns_interleaved() {
        let data = [
            68,                                           // msg id: 'D'
            0, 0, 0, 79,                                  // payload length: 79
            0, 8,                                         // number of columns: 8

            0, 0, 0, 10,                                  // c1: field length: 10
            112, 103, 95, 99, 97, 116, 97, 108, 111, 103, // c1: field - schema specific ("pg_catalog")

            0, 0, 0, 7,                                   // c2: field length: 7
            112, 103, 95, 117, 115, 101, 114,             // c2: field - schema specific ("pg_user")

            0, 0, 0, 4,                                   // c3: field length: 4
            118, 105, 101, 119,                           // c3: field - schema specific ("view")

            0, 0, 0, 4,                                   // c4: field length: 4
            114, 111, 111, 116,                           // c4: field - schema specific ("root")

            0, 0, 0, 9,                                   // c5: field length: 9
            112, 101, 114, 109, 97, 110, 101, 110, 116,   // c5: field - schema specific ("permanent")

            255, 255, 255, 255,                           // c6: NULL column, no value bytes

            0, 0, 0, 7,                                   // c7: field length: 7
            48, 32, 98, 121, 116, 101, 115,               // c7: field - schema specific ("0 bytes")

            255, 255, 255, 255,                           // c8: NULL field, no value bytes
        ];

        let expected = vec![
            Message::DataRow(vec![
                Bytes::from_static(b"pg_catalog"),
                Bytes::from_static(b"pg_user"),
                Bytes::from_static(b"view"),
                Bytes::from_static(b"root"),
                Bytes::from_static(b"permanent"),
                Bytes::from_static(b""),
                Bytes::from_static(b"0 bytes"),
                Bytes::from_static(b""),
            ]),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_empty_query_response() {
        let data = [
            73,         // msg id: 'I'
            0, 0, 0, 4, // payload length: 4
        ];

        let expected = vec![
            Message::EmptyQueryResponse(),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_app_name() {
        let data = [
            83,                                                                           // msg id: 'S'
            0, 0, 0, 26,                                                                  // payload length: 26
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, // cstr: "application_name\0"
            112, 115, 113, 108, 0,                                                        // cstr: "psql\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"application_name"),
                value: Bytes::from_static(b"psql"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_parameter_status_app_name_malformed_cstr() {
        let data = [
            83,                                                                        // msg id: 'S'
            0, 0, 0, 25,                                                               // payload length: 25
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, // malformed cstr
            112, 115, 113, 108, 0,                                                     // cstr: "psql\0"
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_parameter_status_null_size() {
        let data = [
            83,         // msg id: 'S'
            0, 0, 0, 0, // payload length: 0
        ];

        let expected = vec![];
        let remaining = 5;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_client_encoding() {
        let data = [
            83,                                                                        // msg id: 'S'
            0, 0, 0, 25,                                                               // payload length: 25
            99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, // cstr: "client_encoding\0"
            85, 84, 70, 56, 0,                                                         // cstr: "UTF8\0"
        ];
        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"client_encoding"),
                value: Bytes::from_static(b"UTF8"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_date_style() {
        let data = [
            83,                                          // msg id: 'S'
            0, 0, 0, 23,                                 // payload length: 23
            68, 97, 116, 101, 83, 116, 121, 108, 101, 0, // cstr: "DateStyle\0"
            73, 83, 79, 44, 32, 77, 68, 89, 0,           // cstr: "ISO, MDY\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"DateStyle"),
                value: Bytes::from_static(b"ISO, MDY"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_default_transaction_readonly() {
        let data = [
            83,                                                                                                                                         // msg id: 'S'
            0, 0, 0, 38,                                                                                                                                // payload length: 38
            100, 101, 102, 97, 117, 108, 116, 95, 116, 114, 97, 110, 115, 97, 99, 116, 105, 111, 110, 95, 114, 101, 97, 100, 95, 111, 110, 108, 121, 0, // cstr: "default_transaction_read_only\0"
            111, 102, 102, 0,                                                                                                                           // cstr: "off\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"default_transaction_read_only"),
                value: Bytes::from_static(b"off"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_hot_standby() {
        let data = [
            83,                                                                  // msg id: 'S'
            0, 0, 0, 23,                                                         // payload length: 23
            105, 110, 95, 104, 111, 116, 95, 115, 116, 97, 110, 100, 98, 121, 0, // cstr: "in_hot_standby\0"
            111, 102, 102, 0,                                                    // cstr: "off\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"in_hot_standby"),
                value: Bytes::from_static(b"off"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_int_datetimes() {
        let data = [
            83,                                                                                   // msg id: 'S'
            0, 0, 0, 25,                                                                          // payload length: 25
            105, 110, 116, 101, 103, 101, 114, 95, 100, 97, 116, 101, 116, 105, 109, 101, 115, 0, // cstr: "integer_datetimes\0"
            111, 110, 0,                                                                          // cstr: "on\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"integer_datetimes"),
                value: Bytes::from_static(b"on"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_interval_style() {
        let data = [
            83,                                                              // msg id: 'S'
            0, 0, 0, 27,                                                     // payload length: 27
            73, 110, 116, 101, 114, 118, 97, 108, 83, 116, 121, 108, 101, 0, // cstr: "IntervalStyle\0"
            112, 111, 115, 116, 103, 114, 101, 115, 0,                       // cstr: "postgres\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"IntervalStyle"),
                value: Bytes::from_static(b"postgres"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_is_superuser() {
        let data = [
            83,                                                           // msg id: 'S'
            0, 0, 0, 20,                                                  // payload length: 20
            105, 115, 95, 115, 117, 112, 101, 114, 117, 115, 101, 114, 0, // cstr: "is_superuser\0"
            111, 110, 0,                                                  // cstr: "on\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"is_superuser"),
                value: Bytes::from_static(b"on"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_server_encoding() {
        let data = [
            83,                                                                         // msg id: 'S'
            0, 0, 0, 25,                                                                // payload length: 25
            115, 101, 114, 118, 101, 114, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, // cstr: "server_encoding\0"
            85, 84, 70, 56, 0,                                                          // cstr: "UTF8\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"server_encoding"),
                value: Bytes::from_static(b"UTF8"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_server_version() {
        let data = [
            83,                                                                                                                               // msg id: 'S'
            0, 0, 0, 50,                                                                                                                      // payload length: 50
            115, 101, 114, 118, 101, 114, 95, 118, 101, 114, 115, 105, 111, 110, 0,                                                           // cstr: "server_version\0"
            49, 52, 46, 53, 32, 40, 68, 101, 98, 105, 97, 110, 32, 49, 52, 46, 53, 45, 49, 46, 112, 103, 100, 103, 49, 49, 48, 43, 49, 41, 0, // cstr: "14.5 (Debian 14.5-1.pgdg110+1)\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"server_version"),
                value: Bytes::from_static(b"14.5 (Debian 14.5-1.pgdg110+1)"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_session_authorization() {
        let data = [
            83,                                                                                                      // msg id: 'S'
            0, 0, 0, 31,                                                                                             // payload length: 31
            115, 101, 115, 115, 105, 111, 110, 95, 97, 117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110, 0, // cstr: "session_authorization\0"
            114, 111, 111, 116, 0,                                                                                   // cstr: "root\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"session_authorization"),
                value: Bytes::from_static(b"root"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_std_conforming_strings() {
        let data = [
            83,                                                                                                                                  // msg id: 'S'
            0, 0, 0, 35,                                                                                                                         // payload length: 35
            115, 116, 97, 110, 100, 97, 114, 100, 95, 99, 111, 110, 102, 111, 114, 109, 105, 110, 103, 95, 115, 116, 114, 105, 110, 103, 115, 0, // cstr: "standard_conforming_strings\0"
            111, 110, 0,                                                                                                                         // cstr: "on\0"
        ];

        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"standard_conforming_strings"),
                value: Bytes::from_static(b"on"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_timezone() {
        let data = [
            83,                                      // msg id: 'S'
            0, 0, 0, 21,                             // payload length: 21
            84, 105, 109, 101, 90, 111, 110, 101, 0, // cstr: "TimeZone\0"
            69, 116, 99, 47, 85, 84, 67, 0,          // cstr: "Etc/UTC\0"
        ];
        let expected = vec![
            Message::ParameterStatus {
                parameter: Bytes::from_static(b"TimeZone"),
                value: Bytes::from_static(b"Etc/UTC"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_ready_for_query() {
        let data = [
            90,         // msg id: 'Z'
            0, 0, 0, 5, // payload length: 5
            73,         // backend transaction status: 'I': idle ('T': in transaction block; 'E': in failed transaction block)
        ];

        let expected = vec![
            Message::ReadyForQuery(b'I'),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_ready_for_query_null_size() {
        let data = [
            90,         // msg id: 'Z'
            0, 0, 0, 0, // payload length: 0
        ];

        let expected = vec![];
        let remaining = 5;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_ready_for_query_unknown_status() {
        let data = [
            90,         // msg id: 'Z'
            0, 0, 0, 5, // payload length: 5
            0,          // unknown backend transaction status
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_ready_for_query_missing_status() {
        let data = [
            90,         // msg id: 'Z'
            0, 0, 0, 4, // payload length: 5
                        // missing backend transaction status
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_row_description_simple() {
        let data = [
            84,                                     // msg id: 'T'
            0, 0, 0, 33,                            // payload length: 33
            0, 1,                                   // number of fields in a row: 1

            63, 99, 111, 108, 117, 109, 110, 63, 0, // f1: cstr: "?column?\0"
            0, 0, 0, 0,                             // f1: table oid: 0: not identified
            0, 0,                                   // f1: column attr number: 0: not identified
            0, 0, 0, 23,                            // f1: data type oid: 23
            0, 4,                                   // f1: data type size: 4
            255, 255, 255, 255,                     // f1: type modifier - type-specific
            0, 0,                                   // f1: format code: 0: text (1: binary)
        ];

        let expected = vec![
            Message::RowDescription(vec![
                RowDescription {
                    name: Bytes::from_static(b"?column?"),
                    table_oid: 0,
                    column_attr: 0,
                    data_type_oid: 23,
                    data_type_size: 4,
                    type_modifier: -1,
                    format: 0,
                },
            ]),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_row_description_zero_columns() {
        let data = [
            84,         // msg id: 'T'
            0, 0, 0, 6, // payload length: 6
            0, 0,       // number of fields in a row: 0
        ];

        let expected = vec![
            Message::RowDescription(vec![]),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_row_description_missing_field_attr() {
        let data = [
            84,                                     // msg id: 'T'
            0, 0, 0, 31,                            // payload length: 33
            0, 1,                                   // number of fields in a row: 1

            63, 99, 111, 108, 117, 109, 110, 63, 0, // f1: cstr: "?column?\0"
            0, 0, 0, 0,                             // f1: table oid: 0: not identified
            0, 0,                                   // f1: column attr number: 0: not identified
            0, 0, 0, 23,                            // f1: data type oid: 23
            0, 4,                                   // f1: data type size: 4
            255, 255, 255, 255,                     // f1: type modifier - type-specific
                                                    // missing f1: format code
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_row_description_complex() {
        let data = [
            84,                                                             // msg id: 'T'
            0, 0, 0, 216,                                                   // payload length: 216
            0, 8,                                                           // number of fields in a row: 8

            83, 99, 104, 101, 109, 97, 0,                                   // f1: name cstr: "Schema\0"
            0, 0, 10, 55,                                                   // f1: table oid: 2615
            0, 2,                                                           // f1: column attr number: 2
            0, 0, 0, 19,                                                    // f1: data type oid: 19
            0, 64,                                                          // f1: data type size: 64
            255, 255, 255, 255,                                             // f1: type modifier - type-specific
            0, 0,                                                           // f1: format code: 0: text (1: binary)

            78, 97, 109, 101, 0,                                            // f2: name cstr: "Name\0"
            0, 0, 4, 235,                                                   // f2: table oid: 1259
            0, 2,                                                           // f2: column attr number: 2
            0, 0, 0, 19,                                                    // f2: data type oid: 19
            0, 64,                                                          // f2: data type size: 64
            255, 255, 255, 255,                                             // f2: type modifier - type-specific
            0, 0,                                                           // f2: format code: 0: text (1: binary)

            84, 121, 112, 101, 0,                                           // f3: name cstr: "Type\0"
            0, 0, 0, 0,                                                     // f3: table oid: 0: not identified
            0, 0,                                                           // f3: column attr number: not identified
            0, 0, 0, 25,                                                    // f3: data type oid: 25
            255, 255,                                                       // f3: data type size: -1 (variable width)
            255, 255, 255, 255,                                             // f3: type modifier - type-specific
            0, 0,                                                           // f3: format code: 0: text (1: binary)

            79, 119, 110, 101, 114, 0,                                      // f4: name cstr: "Owner\0"
            0, 0, 0, 0,                                                     // f4: table oid: 0: not identified
            0, 0,                                                           // f4: column attr number: 0: not identified
            0, 0, 0, 19,                                                    // f4: data type oid: 19
            0, 64,                                                          // f4: data type size: 64
            255, 255, 255, 255,                                             // f4: type modifier - type-specific
            0, 0,                                                           // f4: format code: 0: text (1: binary)

            80, 101, 114, 115, 105, 115, 116, 101, 110, 99, 101, 0,         // f5: name cstr: "Persistence\0"
            0, 0, 0, 0,                                                     // f5: table oid: 0: not identified
            0, 0,                                                           // f5: column attr number: 0: not identified
            0, 0, 0, 25,                                                    // f5: data type oid: 25
            255, 255,                                                       // f5: data type size: -1 (variable width)
            255, 255, 255, 255,                                             // f5: type modifier - type-specific
            0, 0,                                                           // f5: format code: 0: text (1: binary)

            65, 99, 99, 101, 115, 115, 32, 109, 101, 116, 104, 111, 100, 0, // f6: name cstr: "Access method\0"
            0, 0, 10, 41,                                                   // f6: table oid: 2601
            0, 2,                                                           // f6: column attr number: 2
            0, 0, 0, 19,                                                    // f6: data type oid: 19
            0, 64,                                                          // f6: data type size: 64
            255, 255, 255, 255,                                             // f6: type modifier - type-specific
            0, 0,                                                           // f6: format code: 0: text (1: binary)

            83, 105, 122, 101, 0,                                           // f7: name cstr: "Size\0"
            0, 0, 0, 0,                                                     // f7: table oid: 0: not identified
            0, 0,                                                           // f7: column attr number: 0: not identified
            0, 0, 0, 25,                                                    // f7: data type oid: 25
            255, 255,                                                       // f7: data type size: -1 (variable width)
            255, 255, 255, 255,                                             // f7: type modifier - type-specific
            0, 0,                                                           // f7: format code: 0: text (1: binary)

            68, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 0,         // f8: name cstr: "Description\0"
            0, 0, 0, 0,                                                     // f8: table oid: 0: not identified
            0, 0,                                                           // f8: column attr number: 0: not identified
            0, 0, 0, 25,                                                    // f8: data type oid: 25
            255, 255,                                                       // f8: data type size: -1 (variable width)
            255, 255, 255, 255,                                             // f8: type modifier - type-specific
            0, 0,                                                           // f8: format code: 0: text (1: binary)
        ];

        let expected = vec![
            Message::RowDescription(vec![
                RowDescription {
                    name: Bytes::from_static(b"Schema"),
                    table_oid: 2615,
                    column_attr: 2,
                    data_type_oid: 19,
                    data_type_size: 64,
                    type_modifier: -1,
                    format: 0,
                },
                RowDescription {
                    name: Bytes::from_static(b"Name"),
                    table_oid: 1259,
                    column_attr: 2,
                    data_type_oid: 19,
                    data_type_size: 64,
                    type_modifier: -1,
                    format: 0,
                },
                RowDescription {
                    name: Bytes::from_static(b"Type"),
                    table_oid: 0,
                    column_attr: 0,
                    data_type_oid: 25,
                    data_type_size: -1,
                    type_modifier: -1,
                    format: 0,
                },
                RowDescription {
                    name: Bytes::from_static(b"Owner"),
                    table_oid: 0,
                    column_attr: 0,
                    data_type_oid: 19,
                    data_type_size: 64,
                    type_modifier: -1,
                    format: 0,
                },
                RowDescription {
                    name: Bytes::from_static(b"Persistence"),
                    table_oid: 0,
                    column_attr: 0,
                    data_type_oid: 25,
                    data_type_size: -1,
                    type_modifier: -1,
                    format: 0,
                },
                RowDescription {
                    name: Bytes::from_static(b"Access method"),
                    table_oid: 2601,
                    column_attr: 2,
                    data_type_oid: 19,
                    data_type_size: 64,
                    type_modifier: -1,
                    format: 0,
                },
                RowDescription {
                    name: Bytes::from_static(b"Size"),
                    table_oid: 0,
                    column_attr: 0,
                    data_type_oid: 25,
                    data_type_size: -1,
                    type_modifier: -1,
                    format: 0,
                },
                RowDescription {
                    name: Bytes::from_static(b"Description"),
                    table_oid: 0,
                    column_attr: 0,
                    data_type_oid: 25,
                    data_type_size: -1,
                    type_modifier: -1,
                    format: 0,
                },
            ]),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }
}
