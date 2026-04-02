// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::varint::VarInt;

fn roundtrip(frame: &Frame) {
    let mut buf = [0u8; 1024];
    let written = frame.encode(&mut buf).unwrap();
    let (decoded, consumed) = Frame::decode(&buf[..written]).unwrap();
    assert_eq!(consumed, written, "consumed != written for {frame:?}");
    assert_eq!(&decoded, frame, "roundtrip mismatch for {frame:?}");
}

#[test]
fn padding_roundtrip() {
    roundtrip(&Frame::Padding);
}

#[test]
fn ping_roundtrip() {
    roundtrip(&Frame::Ping);
}

#[test]
fn ack_simple_roundtrip() {
    roundtrip(&Frame::Ack {
        largest_acked: 42,
        ack_delay: 100,
        first_ack_range: 5,
        ack_ranges: vec![],
        ecn: None,
    });
}

#[test]
fn ack_with_ranges_roundtrip() {
    roundtrip(&Frame::Ack {
        largest_acked: 1000,
        ack_delay: 500,
        first_ack_range: 10,
        ack_ranges: vec![
            AckRange {
                gap: 2,
                ack_range: 5,
            },
            AckRange {
                gap: 0,
                ack_range: 3,
            },
        ],
        ecn: None,
    });
}

#[test]
fn ack_ecn_roundtrip() {
    roundtrip(&Frame::Ack {
        largest_acked: 100,
        ack_delay: 0,
        first_ack_range: 0,
        ack_ranges: vec![],
        ecn: Some(EcnCounts {
            ect0: 10,
            ect1: 20,
            ecn_ce: 1,
        }),
    });
}

#[test]
fn reset_stream_roundtrip() {
    roundtrip(&Frame::ResetStream {
        stream_id: 4,
        application_error_code: 0x0100,
        final_size: 2048,
    });
}

#[test]
fn stop_sending_roundtrip() {
    roundtrip(&Frame::StopSending {
        stream_id: 8,
        application_error_code: 42,
    });
}

#[test]
fn crypto_roundtrip() {
    roundtrip(&Frame::Crypto {
        offset: 0,
        data: vec![0x01, 0x00, 0x00, 0xF1, 0x03, 0x03],
    });
}

#[test]
fn new_token_roundtrip() {
    roundtrip(&Frame::NewToken {
        token: vec![0xDE, 0xAD, 0xBE, 0xEF],
    });
}

#[test]
fn stream_all_flags_roundtrip() {
    roundtrip(&Frame::Stream {
        stream_id: 0,
        offset: 100,
        data: b"hello quic".to_vec(),
        flags: StreamFlags {
            has_offset: true,
            has_length: true,
            is_fin: true,
        },
    });
}

#[test]
fn stream_no_offset_no_length() {
    let frame = Frame::Stream {
        stream_id: 4,
        offset: 0,
        data: b"data".to_vec(),
        flags: StreamFlags {
            has_offset: false,
            has_length: false,
            is_fin: false,
        },
    };
    let mut buf = [0u8; 64];
    let written = frame.encode(&mut buf).unwrap();
    let (decoded, consumed) = Frame::decode(&buf[..written]).unwrap();
    assert_eq!(consumed, written);
    if let Frame::Stream {
        stream_id,
        data,
        ..
    } = decoded
    {
        assert_eq!(stream_id, 4);
        assert_eq!(data, b"data");
    } else {
        panic!("expected Stream frame");
    }
}

#[test]
fn max_data_roundtrip() {
    roundtrip(&Frame::MaxData {
        maximum_data: 1_000_000,
    });
}

#[test]
fn max_stream_data_roundtrip() {
    roundtrip(&Frame::MaxStreamData {
        stream_id: 4,
        maximum_stream_data: 65536,
    });
}

#[test]
fn max_streams_roundtrip() {
    roundtrip(&Frame::MaxStreamsBidi {
        maximum_streams: 100,
    });
    roundtrip(&Frame::MaxStreamsUni {
        maximum_streams: 50,
    });
}

#[test]
fn blocked_frames_roundtrip() {
    roundtrip(&Frame::DataBlocked {
        maximum_data: 9999,
    });
    roundtrip(&Frame::StreamDataBlocked {
        stream_id: 12,
        maximum_stream_data: 5000,
    });
    roundtrip(&Frame::StreamsBlockedBidi {
        maximum_streams: 10,
    });
    roundtrip(&Frame::StreamsBlockedUni {
        maximum_streams: 5,
    });
}

#[test]
fn new_connection_id_roundtrip() {
    roundtrip(&Frame::NewConnectionId {
        sequence_number: 1,
        retire_prior_to: 0,
        connection_id: vec![0x01, 0x02, 0x03, 0x04],
        stateless_reset_token: [0xAA; 16],
    });
}

#[test]
fn retire_connection_id_roundtrip() {
    roundtrip(&Frame::RetireConnectionId {
        sequence_number: 5,
    });
}

#[test]
fn path_challenge_response_roundtrip() {
    roundtrip(&Frame::PathChallenge {
        data: [1, 2, 3, 4, 5, 6, 7, 8],
    });
    roundtrip(&Frame::PathResponse {
        data: [8, 7, 6, 5, 4, 3, 2, 1],
    });
}

#[test]
fn connection_close_quic_roundtrip() {
    roundtrip(&Frame::ConnectionCloseQuic {
        error_code: 0x0A,
        frame_type: 0x06,
        reason_phrase: b"flow control error".to_vec(),
    });
}

#[test]
fn connection_close_app_roundtrip() {
    roundtrip(&Frame::ConnectionCloseApp {
        error_code: 42,
        reason_phrase: b"shutting down".to_vec(),
    });
}

#[test]
fn handshake_done_roundtrip() {
    roundtrip(&Frame::HandshakeDone);
}

#[test]
fn stream_flags_bits() {
    let flags = StreamFlags {
        has_offset: true,
        has_length: true,
        is_fin: true,
    };
    assert_eq!(flags.to_type(), 0x0F);
    let decoded = StreamFlags::from_type_bits(0x0F);
    assert_eq!(decoded, flags);

    let none = StreamFlags::from_type_bits(0x08);
    assert!(!none.has_offset);
    assert!(!none.has_length);
    assert!(!none.is_fin);
}

#[test]
fn multiple_frames_in_sequence() {
    let frames = vec![
        Frame::Padding,
        Frame::Ping,
        Frame::Crypto {
            offset: 0,
            data: vec![0x01],
        },
        Frame::Ack {
            largest_acked: 10,
            ack_delay: 0,
            first_ack_range: 0,
            ack_ranges: vec![],
            ecn: None,
        },
    ];
    let mut buf = [0u8; 256];
    let mut total = 0;
    for f in &frames {
        total += f.encode(&mut buf[total..]).unwrap();
    }
    let mut offset = 0;
    for expected in &frames {
        let (decoded, consumed) = Frame::decode(&buf[offset..total]).unwrap();
        assert_eq!(&decoded, expected);
        offset += consumed;
    }
    assert_eq!(offset, total);
}

#[test]
fn empty_buffer_errors() {
    assert!(Frame::decode(&[]).is_err());
}

#[test]
fn unknown_frame_type_errors() {
    let mut buf = [0u8; 4];
    let n = VarInt::from_u32(0xFF).encode(&mut buf).unwrap();
    assert!(Frame::decode(&buf[..n]).is_err());
}

#[test]
fn crypto_frame_truncated_payload_errors() {
    let mut buf = [0u8; 32];
    let mut off = VarInt::new(super::frame_type::CRYPTO).unwrap().encode(&mut buf).unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(10u64).unwrap().encode(&mut buf[off..]).unwrap();
    buf[off] = 0xAB;
    off += 1;
    let err = Frame::decode(&buf[..off]).expect_err("truncated CRYPTO data must fail decode");
    assert!(
        err.to_string().contains("CRYPTO") || err.to_string().contains("truncated"),
        "unexpected error: {err}"
    );
}

#[test]
fn new_token_truncated_errors() {
    let mut buf = [0u8; 16];
    let mut off = VarInt::new(super::frame_type::NEW_TOKEN)
        .unwrap()
        .encode(&mut buf)
        .unwrap();
    off += VarInt::new(5u64).unwrap().encode(&mut buf[off..]).unwrap();
    buf[off] = 1;
    off += 1;
    let err = Frame::decode(&buf[..off]).expect_err("truncated NEW_TOKEN must fail");
    assert!(
        err.to_string().contains("NEW_TOKEN") || err.to_string().contains("truncated"),
        "unexpected error: {err}"
    );
}

#[test]
fn new_connection_id_invalid_cid_length_errors() {
    let mut buf = [0u8; 64];
    let mut off = VarInt::new(super::frame_type::NEW_CONNECTION_ID)
        .unwrap()
        .encode(&mut buf)
        .unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    buf[off] = 21;
    off += 1;
    let err = Frame::decode(&buf[..off]).expect_err("CID length > 20 must fail");
    assert!(
        err.to_string().contains("NEW_CONNECTION_ID") || err.to_string().contains("invalid"),
        "unexpected error: {err}"
    );
}

#[test]
fn path_challenge_truncated_errors() {
    let mut buf = [0u8; 8];
    let n = VarInt::new(super::frame_type::PATH_CHALLENGE)
        .unwrap()
        .encode(&mut buf)
        .unwrap();
    assert_eq!(n, 1);
    let err = Frame::decode(&buf[..n]).expect_err("PATH_CHALLENGE without 8 data bytes must fail");
    assert!(
        err.to_string().contains("PATH") || err.to_string().contains("truncated"),
        "unexpected error: {err}"
    );
}

#[test]
fn connection_close_quic_truncated_reason_errors() {
    let mut buf = [0u8; 32];
    let mut off = VarInt::new(super::frame_type::CONNECTION_CLOSE_QUIC)
        .unwrap()
        .encode(&mut buf)
        .unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(100u64).unwrap().encode(&mut buf[off..]).unwrap();
    let err = Frame::decode(&buf[..off]).expect_err("truncated CONNECTION_CLOSE reason must fail");
    assert!(
        err.to_string().contains("CONNECTION_CLOSE") || err.to_string().contains("truncated"),
        "unexpected error: {err}"
    );
}

#[test]
fn stream_frame_truncated_with_length_errors() {
    let flags = StreamFlags {
        has_offset: false,
        has_length: true,
        is_fin: false,
    };
    let ft = flags.to_type();
    let mut buf = [0u8; 32];
    let mut off = VarInt::new(ft).unwrap().encode(&mut buf).unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(50u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += 2;
    let err = Frame::decode(&buf[..off]).expect_err("STREAM with short payload must fail");
    assert!(
        err.to_string().contains("STREAM") || err.to_string().contains("truncated"),
        "unexpected error: {err}"
    );
}

#[test]
fn encode_rejects_buffer_too_small_for_crypto() {
    let frame = Frame::Crypto {
        offset: 0,
        data: vec![0x01, 0x02, 0x03],
    };
    let mut buf = [0u8; 4];
    let err = frame.encode(&mut buf).expect_err("tiny buffer must not fit CRYPTO frame");
    assert!(
        err.to_string().contains("CRYPTO") || err.to_string().contains("small"),
        "unexpected error: {err}"
    );
}

#[test]
fn padding_byte_not_confused_with_varint_zero() {
    let (f, n) = Frame::decode(&[0x00]).expect("padding decode");
    assert_eq!(n, 1, "PADDING consumes exactly one byte");
    assert_eq!(f, Frame::Padding);
}

#[test]
fn ack_ecn_decode_requires_ecn_varints() {
    let mut buf = [0u8; 32];
    let mut off = VarInt::new(super::frame_type::ACK_ECN).unwrap().encode(&mut buf).unwrap();
    off += VarInt::new(1u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    off += VarInt::new(0u64).unwrap().encode(&mut buf[off..]).unwrap();
    let err = Frame::decode(&buf[..off]).expect_err("ACK_ECN without ECN fields must fail");
    assert!(
        err.to_string().contains("VarInt") || err.to_string().contains("bytes"),
        "unexpected error: {err}"
    );
}
