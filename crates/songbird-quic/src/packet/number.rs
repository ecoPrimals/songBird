// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC packet number encoding, decoding, and expansion (RFC 9000 Appendix A).
//!
//! Packet numbers are truncated to 1-4 bytes on the wire. The receiver must
//! expand them using the largest acknowledged packet number to recover the
//! full 62-bit packet number.

/// Determine the minimum number of bytes needed to encode a packet number
/// such that the receiver can unambiguously recover it given the largest
/// acknowledged packet number.
///
/// RFC 9000 Section 17.1: the sender MUST use a packet number size that is
/// large enough to represent more than twice the number of contiguous
/// unacknowledged packets.
#[must_use]
pub fn encode_pn_length(pn: u64, largest_acked: Option<u64>) -> u8 {
    let num_unacked = match largest_acked {
        Some(la) => pn.saturating_sub(la),
        None => pn + 1,
    };
    // Need enough bits to encode 2 * num_unacked
    let range = 2 * num_unacked;
    if range < (1 << 8) {
        1
    } else if range < (1 << 16) {
        2
    } else if range < (1 << 24) {
        3
    } else {
        4
    }
}

/// Truncate a full packet number to the specified number of bytes.
#[must_use]
pub fn truncate_pn(pn: u64, pn_length: u8) -> u32 {
    let mask = match pn_length {
        1 => 0xFF,
        2 => 0xFFFF,
        3 => 0xFF_FFFF,
        _ => 0xFFFF_FFFF,
    };
    #[expect(clippy::cast_possible_truncation, reason = "masked to at most 32 bits by pn_length")]
    let truncated = (pn & mask) as u32;
    truncated
}

/// Expand a truncated packet number to a full packet number using the
/// largest packet number successfully processed.
///
/// Implements the algorithm from RFC 9000 Appendix A.
#[must_use]
pub fn expand_pn(largest_pn: u64, truncated_pn: u32, pn_nbits: u32) -> u64 {
    let expected_pn = largest_pn + 1;
    let pn_win = 1u64 << pn_nbits;
    let pn_half_win = pn_win / 2;
    let pn_mask = pn_win - 1;

    let candidate_pn = (expected_pn & !pn_mask) | u64::from(truncated_pn);

    if candidate_pn + pn_half_win <= expected_pn && candidate_pn < (1 << 62) - pn_win {
        candidate_pn + pn_win
    } else if candidate_pn > expected_pn + pn_half_win && candidate_pn >= pn_win {
        candidate_pn - pn_win
    } else {
        candidate_pn
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn encode_pn_length_no_ack() {
        assert_eq!(encode_pn_length(0, None), 1);
        // pn=126 → num_unacked=127 → range=254 < 256 → 1 byte
        assert_eq!(encode_pn_length(126, None), 1);
        // pn=127 → num_unacked=128 → range=256 == 256, needs 2 bytes
        assert_eq!(encode_pn_length(127, None), 2);
        assert_eq!(encode_pn_length(128, None), 2);
        // pn=0xFFFF → num_unacked=65536 → range=131072, needs 3 bytes
        assert_eq!(encode_pn_length(0xFFFF, None), 3);
    }

    #[test]
    fn encode_pn_length_with_ack() {
        assert_eq!(encode_pn_length(100, Some(99)), 1);
        assert_eq!(encode_pn_length(200, Some(0)), 2);
        assert_eq!(encode_pn_length(100_000, Some(0)), 3);
    }

    #[test]
    fn truncate_basic() {
        assert_eq!(truncate_pn(0x1234_5678, 1), 0x78);
        assert_eq!(truncate_pn(0x1234_5678, 2), 0x5678);
        assert_eq!(truncate_pn(0x1234_5678, 3), 0x34_5678);
        assert_eq!(truncate_pn(0x1234_5678, 4), 0x1234_5678);
    }

    #[test]
    fn expand_simple_sequential() {
        // Largest seen = 0, truncated = 1, 1-byte (8 bits)
        assert_eq!(expand_pn(0, 1, 8), 1);

        // Largest seen = 100, truncated = 101 (1-byte), should expand to 101
        assert_eq!(expand_pn(100, 101, 8), 101);
    }

    #[test]
    fn expand_wrap_around() {
        // Largest seen = 0xAA82F30E, truncated = 0x9B32 (2 bytes = 16 bits)
        // RFC example from Appendix A
        let full = expand_pn(0xAA82_F30E, 0x9B32, 16);
        assert_eq!(full, 0xAA82_9B32);
    }

    #[test]
    fn expand_near_zero() {
        assert_eq!(expand_pn(0, 0, 8), 0);
        assert_eq!(expand_pn(0, 1, 8), 1);
    }

    #[test]
    fn expand_4_byte_pn() {
        // Large packet numbers with 4-byte encoding
        let pn: u64 = 0x0001_0000_0042;
        let truncated = truncate_pn(pn, 4);
        let expanded = expand_pn(pn - 1, truncated, 32);
        assert_eq!(expanded, pn);
    }

    #[test]
    fn roundtrip_encode_truncate_expand() {
        let test_cases = vec![
            (0u64, None),
            (1, Some(0)),
            (255, Some(200)),
            (1000, Some(990)),
            (70000, Some(69990)),
            (0x1234_5678, Some(0x1234_5670)),
        ];
        for (pn, largest_acked) in test_cases {
            let pn_len = encode_pn_length(pn, largest_acked);
            let truncated = truncate_pn(pn, pn_len);
            let pn_nbits = u32::from(pn_len) * 8;
            let largest = largest_acked.unwrap_or(0);
            let expanded = expand_pn(
                if largest_acked.is_some() {
                    largest
                } else {
                    0
                },
                truncated,
                pn_nbits,
            );
            assert_eq!(
                expanded, pn,
                "roundtrip failed for pn={pn}, la={largest_acked:?}, len={pn_len}"
            );
        }
    }

    #[test]
    fn truncate_identity_small_values() {
        for pn in 0..256u64 {
            assert_eq!(u64::from(truncate_pn(pn, 1)), pn);
        }
    }
}
