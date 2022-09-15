// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

mod common;

mod decode_frontend {

    use bytes::{Bytes, BytesMut};
    use test_log::test;
    use tokio_util::codec::Decoder;

    use fern_protocol_postgresql::codec::frontend::{Codec, Message};

    /// Helper function to ease writing decoding tests.
    fn assert_decode(data: &[u8], expected: &[Message], remaining: usize) {
        let buf = &mut BytesMut::from(data);
        let mut decoded = Vec::new();

        let mut codec = Codec::new();
        codec.startup_complete();

        while let Ok(Some(msg)) = codec.decode(buf) {
            decoded.push(msg);
        }

        assert_eq!(remaining, buf.len(), "remaining bytes in read buffer");
        assert_eq!(expected.len(), decoded.len(), "decoded messages");
        assert_eq!(expected, decoded, "decoded messages");
    }

    #[test]
    #[rustfmt::skip]
    fn valid_execute_no_limit() {
        let data = [
            69,                                                    // msg id: 'E'
            0, 0, 0, 20,                                           // payload length: 20
            112, 111, 114, 116, 97, 108, 95, 110, 97, 109, 101, 0, // cstr: "portal_name\0"
            0, 0, 0, 0                                             // max rows in result: 0 (no limit)
        ];

        let expected = vec![
            Message::Execute {
                portal: Bytes::from_static(b"portal_name"),
                max_rows: 0,
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_execute_unnamed_portal() {
        let data = [
            69,         // msg id: 'E'
            0, 0, 0, 9, // payload length: 9
            0,          // cstr: "\0" (unnamed portal)
            0, 0, 0, 1, // max rows in result: 1
        ];

        let expected = vec![
            Message::Execute {
                portal: Bytes::from_static(b""),
                max_rows: 1,
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_execute_missing_limit() {
        let data = [
            69,                                                    // msg id: 'E'
            0, 0, 0, 16,                                           // payload length: 17
            112, 111, 114, 116, 97, 108, 95, 110, 97, 109, 101, 0, // cstr: "portal_name\0"
                                                                   // missing max rows in result
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_flush() {
        let data = [
            72,         // msg id: 'H'
            0, 0, 0, 4, // payload length: 4
        ];

        let expected = vec![
            Message::Flush(),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sasl_initial_response() {
        let data = [
            112,                                                                    // msg id: 'p' - ! shared id
            0, 0, 0, 54,                                                            // payload length: 54
            83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0,                  // cstr: "SCRAM-SHA-256\0"
            0, 0, 0, 32,                                                            // SASL response length: 32
            110, 44, 44, 110, 61, 44, 114, 61, 79, 103, 111, 110, 89, 82, 110, 108, // SASL response
            48, 52, 97, 100, 103, 66, 51, 54, 83, 112, 76, 113, 111, 85, 52, 117,   // SASL response - continued
        ];

        let expected = vec![
            Message::SASLInitialResponse {
                mecanism: Bytes::from_static(b"SCRAM-SHA-256"),
                response: Bytes::from_static(b"n,,n=,r=OgonYRnl04adgB36SpLqoU4u"),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sasl_initial_response_no_response_data() {
        let data = [
            112,                                                   // msg id: 'p' - ! shared id
            0, 0, 0, 22,                                           // payload length: 22
            83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0, // cstr: "SCRAM-SHA-256\0"
            255, 255, 255, 255,                                    // SASL response length: -1 (no response)
        ];

        let expected = vec![
            Message::SASLInitialResponse {
                mecanism: Bytes::from_static(b"SCRAM-SHA-256"),
                response: Bytes::from_static(b""),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_sasl_initial_response_missing_response_size() {
        let data = [
            112,                                                   // msg id: 'p' - ! shared id
            0, 0, 0, 18,                                           // payload length: 18
            83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0, // cstr: "SCRAM-SHA-256\0"
                                                                   // missing SASL response length
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    // [Postgres protocol message format] defines SASL response length as:
    // _Length of SASL mechanism specific "Initial Client Response" that follows,_
    // _or -1 if there is no Initial Response._
    // A case were SASL response lenght is *0* is therefore considered as valid.
    //
    // [Postgres protocol message format]: https://www.postgresql.org/docs/current/protocol-message-formats.html
    fn valid_sasl_initial_response_null_response_size() {
        let data = [
            112,                                                   // msg id: 'p' - ! shared id
            0, 0, 0, 22,                                           // payload length: 22
            83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0, // cstr: "SCRAM-SHA-256\0"
            0, 0, 0, 0,                                            // SASL response length: 0
                                                                   // missing SASL response
        ];

        let expected = vec![
            Message::SASLInitialResponse {
                mecanism: Bytes::from_static(b"SCRAM-SHA-256"),
                response: Bytes::from_static(b""),
            },
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sasl_response() {
        let data = [
            112,                                                                     // msg id: 'p' - ! shared id
            0, 0, 0, 108,                                                            // payload length: 108
            99, 61, 98, 105, 119, 115, 44, 114, 61, 79, 103, 111, 110, 89, 82, 110,  // SASL response
            108, 48, 52, 97, 100, 103, 66, 51, 54, 83, 112, 76, 113, 111, 85, 52,    // SASL response - cont'd
            117, 83, 97, 52, 73, 110, 52, 115, 81, 122, 105, 90, 86, 106, 87, 50,    // SASL response - cont'd
            97, 112, 122, 66, 48, 48, 108, 111, 79, 44, 112, 61, 84, 99, 48, 117,    // SASL response - cont'd
            87, 90, 76, 66, 71, 73, 108, 110, 51, 97, 120, 69, 50, 108, 51, 66,      // SASL response - cont'd
            54, 84, 102, 122, 101, 119, 87, 113, 115, 79, 101, 99, 57, 71, 113, 103, // SASL response - cont'd
            99, 70, 84, 70, 119, 119, 48, 61,                                        // SASL response - cont'd
        ];

        let expected = vec![
            Message::SASLResponse(
                Bytes::from_static(b"c=biws,r=OgonYRnl04adgB36SpLqoU4uSa4In4sQziZVjW2apzB00loO,p=Tc0uWZLBGIln3axE2l3B6TfzewWqsOec9GqgcFTFww0="),
            ),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_sasl_response_missing_response() {
        let data = [
            112,        // msg id: 'p' - ! shared id
            0, 0, 0, 4, // payload length: 108
                        // missing SASL response
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_query_simple() {
        let data = [
            81,                                       // msg id: 'Q'
            0, 0, 0, 14,                              // payload length: 14
            83, 101, 108, 101, 99, 84, 32, 49, 59, 0, // cstr: "SelecT 1;\0"
        ];

        let expected = vec![
            Message::Query(Bytes::from_static(b"SelecT 1;")),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn invalid_query_simple_missing_null_terminator() {
        let data = [
            81,                                    // msg id: 'Q'
            0, 0, 0, 13,                           // payload length: 13
            83, 101, 108, 101, 99, 84, 32, 49, 59, // invalid cstr: "SelecT 1;"
        ];

        let expected = vec![];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_query_complex() {
        let data = [
            81,                                                                                 // msg id: 'Q'
            0, 0, 4, 54,                                                                        // payload length: 1078
            83, 69, 76, 69, 67, 84, 32, 110, 46, 110, 115, 112, 110, 97, 109, 101,              // cstr: "SELECT n.nspname[...]\0"
            32, 97, 115, 32, 34, 83, 99, 104, 101, 109, 97, 34, 44, 10, 32, 32, 99, 46, 114, 101,
            108, 110, 97, 109, 101, 32, 97, 115, 32, 34, 78, 97, 109, 101, 34, 44, 10, 32, 32, 67,
            65, 83, 69, 32, 99, 46, 114, 101, 108, 107, 105, 110, 100, 32, 87, 72, 69, 78, 32, 39,
            114, 39, 32, 84, 72, 69, 78, 32, 39, 116, 97, 98, 108, 101, 39, 32, 87, 72, 69, 78, 32,
            39, 118, 39, 32, 84, 72, 69, 78, 32, 39, 118, 105, 101, 119, 39, 32, 87, 72, 69, 78,
            32, 39, 109, 39, 32, 84, 72, 69, 78, 32, 39, 109, 97, 116, 101, 114, 105, 97, 108, 105,
            122, 101, 100, 32, 118, 105, 101, 119, 39, 32, 87, 72, 69, 78, 32, 39, 105, 39, 32, 84,
            72, 69, 78, 32, 39, 105, 110, 100, 101, 120, 39, 32, 87, 72, 69, 78, 32, 39, 83, 39,
            32, 84, 72, 69, 78, 32, 39, 115, 101, 113, 117, 101, 110, 99, 101, 39, 32, 87, 72, 69,
            78, 32, 39, 115, 39, 32, 84, 72, 69, 78, 32, 39, 115, 112, 101, 99, 105, 97, 108, 39,
            32, 87, 72, 69, 78, 32, 39, 116, 39, 32, 84, 72, 69, 78, 32, 39, 84, 79, 65, 83, 84,
            32, 116, 97, 98, 108, 101, 39, 32, 87, 72, 69, 78, 32, 39, 102, 39, 32, 84, 72, 69, 78,
            32, 39, 102, 111, 114, 101, 105, 103, 110, 32, 116, 97, 98, 108, 101, 39, 32, 87, 72,
            69, 78, 32, 39, 112, 39, 32, 84, 72, 69, 78, 32, 39, 112, 97, 114, 116, 105, 116, 105,
            111, 110, 101, 100, 32, 116, 97, 98, 108, 101, 39, 32, 87, 72, 69, 78, 32, 39, 73, 39,
            32, 84, 72, 69, 78, 32, 39, 112, 97, 114, 116, 105, 116, 105, 111, 110, 101, 100, 32,
            105, 110, 100, 101, 120, 39, 32, 69, 78, 68, 32, 97, 115, 32, 34, 84, 121, 112, 101,
            34, 44, 10, 32, 32, 112, 103, 95, 99, 97, 116, 97, 108, 111, 103, 46, 112, 103, 95,
            103, 101, 116, 95, 117, 115, 101, 114, 98, 121, 105, 100, 40, 99, 46, 114, 101, 108,
            111, 119, 110, 101, 114, 41, 32, 97, 115, 32, 34, 79, 119, 110, 101, 114, 34, 44, 10,
            32, 32, 67, 65, 83, 69, 32, 99, 46, 114, 101, 108, 112, 101, 114, 115, 105, 115, 116,
            101, 110, 99, 101, 32, 87, 72, 69, 78, 32, 39, 112, 39, 32, 84, 72, 69, 78, 32, 39,
            112, 101, 114, 109, 97, 110, 101, 110, 116, 39, 32, 87, 72, 69, 78, 32, 39, 116, 39,
            32, 84, 72, 69, 78, 32, 39, 116, 101, 109, 112, 111, 114, 97, 114, 121, 39, 32, 87, 72,
            69, 78, 32, 39, 117, 39, 32, 84, 72, 69, 78, 32, 39, 117, 110, 108, 111, 103, 103, 101,
            100, 39, 32, 69, 78, 68, 32, 97, 115, 32, 34, 80, 101, 114, 115, 105, 115, 116, 101,
            110, 99, 101, 34, 44, 10, 32, 32, 97, 109, 46, 97, 109, 110, 97, 109, 101, 32, 97, 115,
            32, 34, 65, 99, 99, 101, 115, 115, 32, 109, 101, 116, 104, 111, 100, 34, 44, 10, 32,
            32, 112, 103, 95, 99, 97, 116, 97, 108, 111, 103, 46, 112, 103, 95, 115, 105, 122, 101,
            95, 112, 114, 101, 116, 116, 121, 40, 112, 103, 95, 99, 97, 116, 97, 108, 111, 103, 46,
            112, 103, 95, 116, 97, 98, 108, 101, 95, 115, 105, 122, 101, 40, 99, 46, 111, 105, 100,
            41, 41, 32, 97, 115, 32, 34, 83, 105, 122, 101, 34, 44, 10, 32, 32, 112, 103, 95, 99,
            97, 116, 97, 108, 111, 103, 46, 111, 98, 106, 95, 100, 101, 115, 99, 114, 105, 112,
            116, 105, 111, 110, 40, 99, 46, 111, 105, 100, 44, 32, 39, 112, 103, 95, 99, 108, 97,
            115, 115, 39, 41, 32, 97, 115, 32, 34, 68, 101, 115, 99, 114, 105, 112, 116, 105, 111,
            110, 34, 10, 70, 82, 79, 77, 32, 112, 103, 95, 99, 97, 116, 97, 108, 111, 103, 46, 112,
            103, 95, 99, 108, 97, 115, 115, 32, 99, 10, 32, 32, 32, 32, 32, 76, 69, 70, 84, 32, 74,
            79, 73, 78, 32, 112, 103, 95, 99, 97, 116, 97, 108, 111, 103, 46, 112, 103, 95, 110,
            97, 109, 101, 115, 112, 97, 99, 101, 32, 110, 32, 79, 78, 32, 110, 46, 111, 105, 100,
            32, 61, 32, 99, 46, 114, 101, 108, 110, 97, 109, 101, 115, 112, 97, 99, 101, 10, 32,
            32, 32, 32, 32, 76, 69, 70, 84, 32, 74, 79, 73, 78, 32, 112, 103, 95, 99, 97, 116, 97,
            108, 111, 103, 46, 112, 103, 95, 97, 109, 32, 97, 109, 32, 79, 78, 32, 97, 109, 46,
            111, 105, 100, 32, 61, 32, 99, 46, 114, 101, 108, 97, 109, 10, 87, 72, 69, 82, 69, 32,
            99, 46, 114, 101, 108, 107, 105, 110, 100, 32, 73, 78, 32, 40, 39, 114, 39, 44, 39,
            112, 39, 44, 39, 118, 39, 44, 39, 109, 39, 44, 39, 83, 39, 44, 39, 102, 39, 44, 39, 39,
            41, 10, 32, 32, 32, 32, 32, 32, 65, 78, 68, 32, 110, 46, 110, 115, 112, 110, 97, 109,
            101, 32, 60, 62, 32, 39, 112, 103, 95, 99, 97, 116, 97, 108, 111, 103, 39, 10, 32, 32,
            32, 32, 32, 32, 65, 78, 68, 32, 110, 46, 110, 115, 112, 110, 97, 109, 101, 32, 33, 126,
            32, 39, 94, 112, 103, 95, 116, 111, 97, 115, 116, 39, 10, 32, 32, 32, 32, 32, 32, 65,
            78, 68, 32, 110, 46, 110, 115, 112, 110, 97, 109, 101, 32, 60, 62, 32, 39, 105, 110,
            102, 111, 114, 109, 97, 116, 105, 111, 110, 95, 115, 99, 104, 101, 109, 97, 39, 10, 32,
            32, 65, 78, 68, 32, 112, 103, 95, 99, 97, 116, 97, 108, 111, 103, 46, 112, 103, 95,
            116, 97, 98, 108, 101, 95, 105, 115, 95, 118, 105, 115, 105, 98, 108, 101, 40, 99, 46,
            111, 105, 100, 41, 10, 79, 82, 68, 69, 82, 32, 66, 89, 32, 49, 44, 50, 59, 0,
        ];

        let sql_query = r#"SELECT n.nspname as "Schema",
  c.relname as "Name",
  CASE c.relkind WHEN 'r' THEN 'table' WHEN 'v' THEN 'view' WHEN 'm' THEN 'materialized view' WHEN 'i' THEN 'index' WHEN 'S' THEN 'sequence' WHEN 's' THEN 'special' WHEN 't' THEN 'TOAST table' WHEN 'f' THEN 'foreign table' WHEN 'p' THEN 'partitioned table' WHEN 'I' THEN 'partitioned index' END as "Type",
  pg_catalog.pg_get_userbyid(c.relowner) as "Owner",
  CASE c.relpersistence WHEN 'p' THEN 'permanent' WHEN 't' THEN 'temporary' WHEN 'u' THEN 'unlogged' END as "Persistence",
  am.amname as "Access method",
  pg_catalog.pg_size_pretty(pg_catalog.pg_table_size(c.oid)) as "Size",
  pg_catalog.obj_description(c.oid, 'pg_class') as "Description"
FROM pg_catalog.pg_class c
     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
     LEFT JOIN pg_catalog.pg_am am ON am.oid = c.relam
WHERE c.relkind IN ('r','p','v','m','S','f','')
      AND n.nspname <> 'pg_catalog'
      AND n.nspname !~ '^pg_toast'
      AND n.nspname <> 'information_schema'
  AND pg_catalog.pg_table_is_visible(c.oid)
ORDER BY 1,2;"#;

        let expected = vec![
            Message::Query(Bytes::from_static(sql_query.as_bytes())),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sync() {
        let data = [
            83,         // msg id: 'S'
            0, 0, 0, 4, // payload length: 4
        ];

        let expected = vec![
            Message::Sync(),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_terminate() {
        let data = [
            88,         // msg id: 'X'
            0, 0, 0, 4, // payload length: 4
        ];

        let expected = vec![
            Message::Terminate(),
        ];
        let remaining = 0;

        assert_decode(&data[..], &expected, remaining);
    }
}
