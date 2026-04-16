// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! RFC 9001 Section 5.4: Header protection.
//!
//! Header protection masks parts of the packet header to prevent middleboxes
//! from reading the packet number or other header fields. The mask is derived
//! from a sample of the encrypted payload using `AES-ECB` or `ChaCha20` depending
//! on the cipher suite.

use super::provider::{QuicCipherSuite, SecurityQuicCrypto};
use crate::error::{QuicError, Result};
use crate::packet::header::is_long_header;

/// Apply header protection to a packet (before sending).
///
/// This modifies the first byte and the packet number bytes in-place.
///
/// `pn_offset` is the byte offset of the packet number within `packet`.
/// `pn_length` is the number of packet number bytes (1–4).
///
/// # Errors
///
/// Returns [`QuicError::Crypto`] if the packet buffer is too short for the header-protection sample.
pub async fn apply_header_protection(
    crypto: &SecurityQuicCrypto,
    suite: QuicCipherSuite,
    hp_key: &[u8],
    packet: &mut [u8],
    pn_offset: usize,
    pn_length: usize,
) -> Result<()> {
    let sample_offset = pn_offset + 4; // RFC 9001: sample starts 4 bytes after PN offset
    if sample_offset + suite.hp_sample_len() > packet.len() {
        return Err(QuicError::Crypto("Packet too short for HP sample".into()));
    }

    let sample: Vec<u8> = packet[sample_offset..sample_offset + suite.hp_sample_len()].to_vec();
    let mask = crypto.header_protection_mask(suite, hp_key, &sample).await?;

    apply_mask(packet, mask, pn_offset, pn_length);
    Ok(())
}

/// Remove header protection from a received packet.
///
/// Since the mask is `XORed`, removal uses the same operation as application.
/// However, we first need to determine the PN length, which requires
/// partially unmasking the first byte.
///
/// Returns the actual packet number length after unmasking.
///
/// # Errors
///
/// Returns [`QuicError::Crypto`] if the packet buffer is too short for the header-protection sample.
pub async fn remove_header_protection(
    crypto: &SecurityQuicCrypto,
    suite: QuicCipherSuite,
    hp_key: &[u8],
    packet: &mut [u8],
    pn_offset: usize,
) -> Result<usize> {
    let sample_offset = pn_offset + 4;
    if sample_offset + suite.hp_sample_len() > packet.len() {
        return Err(QuicError::Crypto("Packet too short for HP sample".into()));
    }

    let sample: Vec<u8> = packet[sample_offset..sample_offset + suite.hp_sample_len()].to_vec();
    let mask = crypto.header_protection_mask(suite, hp_key, &sample).await?;

    // Unmask first byte to determine PN length
    let first_byte = if is_long_header(packet[0]) {
        packet[0] ^ (mask[0] & 0x0F)
    } else {
        packet[0] ^ (mask[0] & 0x1F)
    };
    let pn_length = ((first_byte & 0x03) + 1) as usize;

    // Apply the full mask
    apply_mask(packet, mask, pn_offset, pn_length);

    Ok(pn_length)
}

/// XOR the mask into the packet header.
///
/// RFC 9001 Section 5.4.1:
/// - Long headers: `mask[0]` `XORed` with bits 0–3 of the first byte (4 bits)
/// - Short headers: `mask[0]` `XORed` with bits 0–4 of the first byte (5 bits)
/// - `mask[1..1+pn_length]` `XORed` with the packet number bytes
fn apply_mask(packet: &mut [u8], mask: [u8; 5], pn_offset: usize, pn_length: usize) {
    if is_long_header(packet[0]) {
        packet[0] ^= mask[0] & 0x0F;
    } else {
        packet[0] ^= mask[0] & 0x1F;
    }

    for i in 0..pn_length {
        if pn_offset + i < packet.len() {
            packet[pn_offset + i] ^= mask[1 + i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_mask_long_header() {
        // Long header: form bit = 1 (0x80 set)
        let mut packet = vec![0xC3, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x42];
        let mask = [0x0A, 0x11, 0x22, 0x33, 0x44];
        let pn_offset = 7;
        let pn_length = 1;

        let original_first = packet[0];
        let original_pn = packet[pn_offset];

        apply_mask(&mut packet, mask, pn_offset, pn_length);

        // Long header: only low 4 bits of mask[0] applied
        assert_eq!(packet[0], original_first ^ (mask[0] & 0x0F));
        // PN byte XORed with mask[1]
        assert_eq!(packet[pn_offset], original_pn ^ mask[1]);
    }

    #[test]
    fn apply_mask_short_header() {
        // Short header: form bit = 0
        let mut packet = vec![0x43, 0xAA, 0xBB, 0xCC, 0xDD, 0x00, 0x01];
        let mask = [0x1F, 0xAA, 0xBB, 0xCC, 0xDD];
        let pn_offset = 5;
        let pn_length = 2;

        let original_first = packet[0];

        apply_mask(&mut packet, mask, pn_offset, pn_length);

        // Short header: low 5 bits of mask[0]
        assert_eq!(packet[0], original_first ^ (mask[0] & 0x1F));
        assert_eq!(packet[5], 0xAA);
        assert_eq!(packet[6], 0x01 ^ 0xBB);
    }

    #[test]
    fn mask_is_self_inverse() {
        let mut packet = vec![0xC0, 0x00, 0x00, 0x00, 0x01, 0x08, 0x00, 0x42, 0x00];
        let original = packet.clone();
        let mask = [0x05, 0x12, 0x34, 0x56, 0x78];
        let pn_offset = 7;
        let pn_length = 2;

        apply_mask(&mut packet, mask, pn_offset, pn_length);
        assert_ne!(packet, original);

        apply_mask(&mut packet, mask, pn_offset, pn_length);
        assert_eq!(packet, original);
    }
}
