// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! RFC 9001 Section 5.2: Initial secret and key derivation.
//!
//! Initial packets are protected using keys derived from the client's
//! Destination Connection ID. This is deterministic (not secret) and exists
//! to bootstrap the TLS handshake.

use super::provider::{QuicCipherSuite, QuicCryptoProvider};
use crate::error::Result;

/// QUIC v1 Initial salt (RFC 9001 Section 5.2).
pub const QUIC_V1_INITIAL_SALT: &[u8] = &[
    0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17, 0x9a, 0xe6, 0xa4, 0xc8, 0x0c, 0xad,
    0xcc, 0xbb, 0x7f, 0x0a,
];

/// Initial cipher suite is always AES-128-GCM-SHA256.
pub const INITIAL_CIPHER_SUITE: QuicCipherSuite = QuicCipherSuite::Aes128Gcm;

/// Derived key set for one direction (client or server).
#[derive(Debug, Clone)]
pub struct DirectionalKeys {
    /// AEAD key.
    pub key: Vec<u8>,
    /// AEAD IV (nonce base).
    pub iv: Vec<u8>,
    /// Header protection key.
    pub hp_key: Vec<u8>,
}

/// Both client and server Initial keys.
#[derive(Debug, Clone)]
pub struct InitialKeys {
    /// Client-side keys (for protecting client Initial packets).
    pub client: DirectionalKeys,
    /// Server-side keys (for protecting server Initial packets).
    pub server: DirectionalKeys,
}

/// Derive Initial keys from the Destination Connection ID.
///
/// # Errors
///
/// Returns [`QuicError`](crate::error::QuicError) when `HKDF` or key derivation fails.
///
/// RFC 9001 Section 5.2:
/// ```text
/// initial_salt = 0x38762cf7f55934b34d179ae6a4c80cadccbb7f0a
/// initial_secret = HKDF-Extract(initial_salt, client_dst_connection_id)
/// client_initial_secret = HKDF-Expand-Label(initial_secret, "client in", "", 32)
/// server_initial_secret = HKDF-Expand-Label(initial_secret, "server in", "", 32)
/// ```
pub async fn derive_initial_keys(
    crypto: &dyn QuicCryptoProvider,
    dcid: &[u8],
) -> Result<InitialKeys> {
    let initial_secret = crypto.hkdf_extract(QUIC_V1_INITIAL_SALT, dcid).await?;

    let client_initial_secret = hkdf_expand_label(
        crypto,
        &initial_secret,
        b"client in",
        &[],
        INITIAL_CIPHER_SUITE.hash_len(),
    )
    .await?;

    let server_initial_secret = hkdf_expand_label(
        crypto,
        &initial_secret,
        b"server in",
        &[],
        INITIAL_CIPHER_SUITE.hash_len(),
    )
    .await?;

    let client = derive_directional_keys(crypto, &client_initial_secret).await?;
    let server = derive_directional_keys(crypto, &server_initial_secret).await?;

    Ok(InitialKeys {
        client,
        server,
    })
}

/// Derive AEAD key, IV, and HP key from a traffic secret.
///
/// # Errors
///
/// Returns [`QuicError`](crate::error::QuicError) when HKDF expansion fails.
///
/// RFC 9001 Section 5.1:
/// ```text
/// quic key = HKDF-Expand-Label(secret, "quic key", "", key_len)
/// quic iv  = HKDF-Expand-Label(secret, "quic iv",  "", iv_len)
/// quic hp  = HKDF-Expand-Label(secret, "quic hp",  "", hp_key_len)
/// ```
pub async fn derive_directional_keys(
    crypto: &dyn QuicCryptoProvider,
    secret: &[u8],
) -> Result<DirectionalKeys> {
    let key =
        hkdf_expand_label(crypto, secret, b"quic key", &[], INITIAL_CIPHER_SUITE.key_len()).await?;

    let iv =
        hkdf_expand_label(crypto, secret, b"quic iv", &[], INITIAL_CIPHER_SUITE.iv_len()).await?;

    let hp_key =
        hkdf_expand_label(crypto, secret, b"quic hp", &[], INITIAL_CIPHER_SUITE.hp_key_len())
            .await?;

    Ok(DirectionalKeys {
        key,
        iv,
        hp_key,
    })
}

/// `HKDF-Expand-Label` as defined in RFC 8446 Section 7.1, used by QUIC.
///
/// # Errors
///
/// Returns [`QuicError`](crate::error::QuicError) when HKDF expansion fails.
///
/// ```text
/// HKDF-Expand-Label(Secret, Label, Context, Length) =
///     HKDF-Expand(Secret, HkdfLabel, Length)
///
/// struct {
///     uint16 length = Length;
///     opaque label<7..255> = "tls13 " + Label;
///     opaque context<0..255> = Context;
/// } HkdfLabel;
/// ```
pub async fn hkdf_expand_label(
    crypto: &dyn QuicCryptoProvider,
    secret: &[u8],
    label: &[u8],
    context: &[u8],
    length: usize,
) -> Result<Vec<u8>> {
    let full_label = build_hkdf_label(label, context, length);
    crypto.hkdf_expand(secret, &full_label, length).await
}

/// Build the `HkdfLabel` structure for `HKDF-Expand-Label`.
fn build_hkdf_label(label: &[u8], context: &[u8], length: usize) -> Vec<u8> {
    let tls13_label = [b"tls13 ", label].concat();

    let mut info = Vec::with_capacity(2 + 1 + tls13_label.len() + 1 + context.len());
    // uint16 length
    #[expect(
        clippy::cast_possible_truncation,
        reason = "HKDF label length matches hkdf_expand length parameter (<= 255 for QUIC)"
    )]
    let length_u16 = length as u16;
    info.extend_from_slice(&length_u16.to_be_bytes());
    // opaque label<7..255>
    #[expect(
        clippy::cast_possible_truncation,
        reason = "TLS label length is bounded by HKDF-Expand-Label construction"
    )]
    let label_len = tls13_label.len() as u8;
    info.push(label_len);
    info.extend_from_slice(&tls13_label);
    // opaque context<0..255>
    #[expect(
        clippy::cast_possible_truncation,
        reason = "HKDF context length is at most 255 bytes per RFC 8446"
    )]
    let ctx_len = context.len() as u8;
    info.push(ctx_len);
    info.extend_from_slice(context);

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_salt_is_correct_length() {
        assert_eq!(QUIC_V1_INITIAL_SALT.len(), 20);
    }

    #[test]
    fn build_hkdf_label_structure() {
        let label = build_hkdf_label(b"client in", &[], 32);
        // uint16 length = 32
        assert_eq!(label[0], 0);
        assert_eq!(label[1], 32);
        // label length byte: "tls13 client in" = 15
        assert_eq!(label[2], 15);
        assert_eq!(&label[3..18], b"tls13 client in");
        // context length = 0
        assert_eq!(label[18], 0);
        assert_eq!(label.len(), 19);
    }

    #[test]
    fn build_hkdf_label_with_context() {
        let ctx = [0xAA, 0xBB];
        let label = build_hkdf_label(b"quic key", &ctx, 16);
        // uint16 length = 16
        assert_eq!(u16::from_be_bytes([label[0], label[1]]), 16);
        // "tls13 quic key" = 14 bytes
        assert_eq!(label[2], 14);
        assert_eq!(&label[3..17], b"tls13 quic key");
        // context
        assert_eq!(label[17], 2);
        assert_eq!(&label[18..20], &ctx);
    }

    #[test]
    fn initial_cipher_suite_properties() {
        assert_eq!(INITIAL_CIPHER_SUITE.key_len(), 16);
        assert_eq!(INITIAL_CIPHER_SUITE.iv_len(), 12);
        assert_eq!(INITIAL_CIPHER_SUITE.hash_len(), 32);
    }
}
