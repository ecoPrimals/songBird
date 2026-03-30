// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC packet protection: AEAD encrypt/decrypt with packet-number-derived nonces.
//!
//! RFC 9001 Section 5.3: each QUIC packet payload is protected with an
//! AEAD algorithm. The nonce is constructed by XORing the IV with the
//! packet number (left-padded to IV length).

use super::initial_keys::DirectionalKeys;
use super::provider::{QuicCipherSuite, QuicCryptoProvider};
use crate::error::Result;

/// Construct the AEAD nonce from the IV and packet number.
///
/// RFC 9001 Section 5.3:
/// The packet number is left-padded with zeros to the IV length,
/// then XORed with the IV.
#[must_use]
pub fn build_nonce(iv: &[u8], packet_number: u64) -> Vec<u8> {
    let mut nonce = iv.to_vec();
    let pn_bytes = packet_number.to_be_bytes();
    let offset = nonce.len().saturating_sub(8);
    for i in 0..8 {
        if offset + i < nonce.len() {
            nonce[offset + i] ^= pn_bytes[i];
        }
    }
    nonce
}

/// Encrypt a QUIC packet payload.
///
/// The AAD (additional authenticated data) is the QUIC packet header
/// (from the first byte through the packet number, inclusive).
pub async fn protect_payload(
    crypto: &dyn QuicCryptoProvider,
    suite: QuicCipherSuite,
    keys: &DirectionalKeys,
    packet_number: u64,
    header: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    let nonce = build_nonce(&keys.iv, packet_number);
    crypto
        .aead_encrypt(suite, &keys.key, &nonce, plaintext, header)
        .await
}

/// Decrypt a QUIC packet payload.
///
/// The AAD must be the same header bytes used during encryption.
pub async fn unprotect_payload(
    crypto: &dyn QuicCryptoProvider,
    suite: QuicCipherSuite,
    keys: &DirectionalKeys,
    packet_number: u64,
    header: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let nonce = build_nonce(&keys.iv, packet_number);
    crypto
        .aead_decrypt(suite, &keys.key, &nonce, ciphertext, header)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_nonce_zero_pn() {
        let iv = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C];
        let nonce = build_nonce(&iv, 0);
        assert_eq!(nonce, iv);
    }

    #[test]
    fn build_nonce_xor() {
        let iv = vec![0x00; 12];
        let nonce = build_nonce(&iv, 1);
        // PN=1 → last byte of 8-byte BE = 0x01, XORed with iv[11]=0x00 → 0x01
        assert_eq!(nonce[11], 0x01);
        assert_eq!(nonce[0..11], [0x00; 11]);
    }

    #[test]
    fn build_nonce_large_pn() {
        let iv = vec![0xFF; 12];
        let pn: u64 = 0x0102_0304_0506_0708;
        let nonce = build_nonce(&iv, pn);
        let pn_bytes = pn.to_be_bytes();
        // bytes 4..12 of nonce = iv[4..12] XOR pn_bytes[0..8]
        for i in 0..8 {
            assert_eq!(nonce[4 + i], 0xFF ^ pn_bytes[i]);
        }
        // first 4 bytes unchanged (no overlap with 8-byte PN)
        assert_eq!(&nonce[..4], &[0xFF; 4]);
    }

    #[test]
    fn rfc_nonce_construction_example() {
        // RFC 9001 uses a 12-byte IV and up to 62-bit PN
        let iv = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B,
        ];
        let pn = 42u64;
        let nonce = build_nonce(&iv, pn);
        assert_eq!(nonce.len(), 12);
        // Only the last byte should differ (pn=42 fits in 1 byte)
        assert_eq!(nonce[11], 0x0B ^ 42);
        assert_eq!(&nonce[..11], &iv[..11]);
    }
}
