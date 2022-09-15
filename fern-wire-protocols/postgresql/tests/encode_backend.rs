// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

mod common;

mod encode_backend {

    use bytes::{Bytes, BytesMut};
    use test_log::test;
    use tokio_util::codec::{Decoder, Encoder};

    use fern_protocol_postgresql::codec::backend::{Codec, Message, RowDescription};

    /// Helper function to ease writing encoding tests.
    fn assert_encode(msg: Message) {
        let mut codec = Codec::new();
        let buf = &mut BytesMut::new();

        codec.encode(msg.clone(), buf).unwrap();
        assert_eq!(Some(msg), codec.decode(buf).unwrap());
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_ok() {
        let msg = Message::AuthenticationOk();

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_sasl() {
        let msg = Message::AuthenticationSASL(
            Bytes::from_static(b"SCRAM-SHA-256"),
        );

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_sasl_continue() {
        let msg = Message::AuthenticationSASLContinue(
            Bytes::from_static(b"r=OgonYRnl04adgB36SpLqoU4uSa4In4sQziZVjW2apzB00loO,s=5xcLn0p1FZY4wjrO2sI17w==,i=4096"),
        );

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_authentication_sasl_final() {
        let msg = Message::AuthenticationSASLFinal(
            Bytes::from_static(b"v=kWVc+uM8icA0mjBjIVg07qb8NlrMRpKRHWrFbcdQJow="),
        );

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_backend_key_data() {
        let msg = Message::BackendKeyData {
            process: 80,
            secret_key: 4009251115,
        };

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_command_complete() {
        let msg = Message::CommandComplete(
            Bytes::from_static(b"SELECT 1"),
        );

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_simple() {
        let msg = Message::DataRow(vec![
            Bytes::from_static(b"1"),
        ]);

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_zero_columns() {
        let msg = Message::DataRow(vec![]);

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_complex_null_columns_ending() {
        let msg = Message::DataRow(vec![
            Bytes::from_static(b"pg_catalog"),
            Bytes::from_static(b"pg_aggregate"),
            Bytes::from_static(b"table"),
            Bytes::from_static(b"root"),
            Bytes::from_static(b"permanent"),
            Bytes::from_static(b"heap"),
            Bytes::from_static(b"56 kB"),
            Bytes::from_static(b""),
        ]);

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_data_row_complex_null_columns_interleaved() {
        let msg = Message::DataRow(vec![
            Bytes::from_static(b"pg_catalog"),
            Bytes::from_static(b"pg_user"),
            Bytes::from_static(b"view"),
            Bytes::from_static(b"root"),
            Bytes::from_static(b"permanent"),
            Bytes::from_static(b""),
            Bytes::from_static(b"0 bytes"),
            Bytes::from_static(b""),
        ]);

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_empty_query_response() {
        let msg = Message::EmptyQueryResponse();

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_parameter_status_app_name() {
        let msg = Message::ParameterStatus {
            parameter: Bytes::from_static(b"application_name"),
            value: Bytes::from_static(b"psql"),
        };

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_ready_for_query() {
        let msg = Message::ReadyForQuery(b'I');

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_row_description_simple() {
        let msg = Message::RowDescription(vec![
            RowDescription {
                name: Bytes::from_static(b"?column?"),
                table_oid: 0,
                column_attr: 0,
                data_type_oid: 23,
                data_type_size: 4,
                type_modifier: -1,
                format: 0,
            },
        ]);

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_row_description_zero_columns() {
        let msg = Message::RowDescription(vec![]);

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_row_description_complex() {
        let msg =  Message::RowDescription(vec![
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
        ]);

        assert_encode(msg);
    }
}
