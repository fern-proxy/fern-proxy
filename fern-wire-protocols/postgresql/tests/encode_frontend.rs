// SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
// SPDX-License-Identifier: Apache-2.0

mod common;

mod encode_frontend {

    use bytes::{Bytes, BytesMut};
    use test_log::test;
    use tokio_util::codec::{Decoder, Encoder};

    use fern_protocol_postgresql::codec::frontend::{Codec, Message};

    /// Helper function to ease writing encoding tests.
    fn assert_encode(msg: Message) {
        let mut codec = Codec::new();
        let buf = &mut BytesMut::new();

        codec.startup_complete();
        codec.encode(msg.clone(), buf).unwrap();
        assert_eq!(Some(msg), codec.decode(buf).unwrap());
    }

    #[test]
    #[rustfmt::skip]
    fn valid_execute_no_limit() {
        let msg = Message::Execute {
            portal: Bytes::from_static(b"portal_name"),
            max_rows: 0,
        };

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_execute_unnamed_portal() {
        let msg = Message::Execute {
            portal: Bytes::from_static(b""),
            max_rows: 1,
        };

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_flush() {
        let msg = Message::Flush();

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_query_simple() {
        let msg = Message::Query(Bytes::from_static(b"SelecT 1;"));

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sasl_initial_response() {
        let msg = Message::SASLInitialResponse {
            mecanism: Bytes::from_static(b"SCRAM-SHA-256"),
            response: Bytes::from_static(b"n,,n=,r=OgonYRnl04adgB36SpLqoU4u"),
        };

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sasl_initial_response_no_response_data() {
        let msg = Message::SASLInitialResponse {
            mecanism: Bytes::from_static(b"SCRAM-SHA-256"),
            response: Bytes::from_static(b""),
        };

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sasl_response() {
        let msg = Message::SASLResponse(
            Bytes::from_static(b"c=biws,r=OgonYRnl04adgB36SpLqoU4uSa4In4sQziZVjW2apzB00loO,p=Tc0uWZLBGIln3axE2l3B6TfzewWqsOec9GqgcFTFww0="),
        );

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_sync() {
        let msg = Message::Sync();

        assert_encode(msg);
    }

    #[test]
    #[rustfmt::skip]
    fn valid_terminate() {
        let msg = Message::Terminate();

        assert_encode(msg);
    }
}
