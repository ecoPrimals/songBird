// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Pure-Rust self-signed Ed25519 certificate generation for inter-primal QUIC.
//!
//! Replaces `rcgen` (which pulled `ring` transitively) with direct DER construction
//! using `ed25519-dalek`. The certificates are self-signed and minimal — real identity
//! verification happens via `security provider` lineage at the application layer.

use ed25519_dalek::{Signer, SigningKey};
use rand::RngCore;

/// OID for Ed25519 (1.3.101.112)
const OID_ED25519: &[u8] = &[0x2b, 0x65, 0x70];

/// OID for commonName (2.5.4.3)
const OID_CN: &[u8] = &[0x55, 0x04, 0x03];

/// OID for subjectAltName (2.5.29.17)
const OID_SAN: &[u8] = &[0x55, 0x1d, 0x11];

/// Generate a self-signed Ed25519 certificate and PKCS#8 private key.
///
/// Returns `(cert_der, private_key_pkcs8_der)`.
#[must_use]
pub fn generate_self_signed_ed25519(domain: &str) -> (Vec<u8>, Vec<u8>) {
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key();

    // Serial number: first 16 bytes of the public key (deterministic, unique per keypair)
    let serial = &public_key.as_bytes()[..16];

    let tbs = build_tbs_certificate(serial, domain, public_key.as_bytes());
    let signature = signing_key.sign(&tbs);

    let cert_der = der_sequence(&[
        &tbs,
        &der_sequence(&[&der_oid(OID_ED25519)]),
        &der_bit_string(&signature.to_bytes()),
    ]);

    let priv_key_der = ed25519_pkcs8_der(signing_key.as_bytes());

    (cert_der, priv_key_der)
}

fn build_tbs_certificate(serial: &[u8], domain: &str, public_key: &[u8; 32]) -> Vec<u8> {
    let version = der_explicit(0, &der_integer(&[2])); // v3
    let serial_num = der_integer(serial);
    let sig_alg = der_sequence(&[&der_oid(OID_ED25519)]);
    let name = build_name(domain);
    let validity = der_sequence(&[
        &der_generalized_time("20240101000000Z"),
        &der_generalized_time("20990101000000Z"),
    ]);
    let spki =
        der_sequence(&[&der_sequence(&[&der_oid(OID_ED25519)]), &der_bit_string(public_key)]);

    // SubjectAltName extension: dNSName
    let san_value = der_sequence(&[&der_implicit(2, domain.as_bytes())]);
    let san_ext = der_sequence(&[&der_oid(OID_SAN), &der_octet_string(&san_value)]);
    let extensions = der_explicit(3, &der_sequence(&[&san_ext]));

    der_sequence(&[&version, &serial_num, &sig_alg, &name, &validity, &name, &spki, &extensions])
}

fn build_name(cn: &str) -> Vec<u8> {
    let attr = der_sequence(&[&der_oid(OID_CN), &der_utf8_string(cn)]);
    let rdn = der_set(&[&attr]);
    der_sequence(&[&rdn])
}

/// Ed25519 private key in PKCS#8 DER format.
fn ed25519_pkcs8_der(secret: &[u8; 32]) -> Vec<u8> {
    let mut der = Vec::with_capacity(48);
    der.extend_from_slice(&[0x30, 0x2e]); // SEQUENCE (46 bytes)
    der.extend_from_slice(&[0x02, 0x01, 0x00]); // INTEGER 0 (version)
    der.extend_from_slice(&[0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70]); // AlgorithmIdentifier
    der.extend_from_slice(&[0x04, 0x22, 0x04, 0x20]); // OCTET STRING { OCTET STRING { key } }
    der.extend_from_slice(secret);
    der
}

// --- DER encoding primitives ---

fn der_tag_length_value(tag: u8, content: &[u8]) -> Vec<u8> {
    let len = content.len();
    let mut out = Vec::with_capacity(1 + length_bytes(len) + len);
    out.push(tag);
    push_length(&mut out, len);
    out.extend_from_slice(content);
    out
}

fn der_sequence(items: &[&[u8]]) -> Vec<u8> {
    let content = concat_slices(items);
    der_tag_length_value(0x30, &content)
}

fn der_set(items: &[&[u8]]) -> Vec<u8> {
    let content = concat_slices(items);
    der_tag_length_value(0x31, &content)
}

fn der_integer(value: &[u8]) -> Vec<u8> {
    // Ensure leading zero if high bit set (DER integer encoding)
    if value.first().is_some_and(|&b| b & 0x80 != 0) {
        let mut padded = Vec::with_capacity(1 + value.len());
        padded.push(0x00);
        padded.extend_from_slice(value);
        der_tag_length_value(0x02, &padded)
    } else {
        der_tag_length_value(0x02, value)
    }
}

fn der_oid(oid_bytes: &[u8]) -> Vec<u8> {
    der_tag_length_value(0x06, oid_bytes)
}

fn der_utf8_string(s: &str) -> Vec<u8> {
    der_tag_length_value(0x0c, s.as_bytes())
}

fn der_bit_string(data: &[u8]) -> Vec<u8> {
    // BIT STRING: leading byte is the number of unused bits (always 0 for us)
    let mut content = Vec::with_capacity(1 + data.len());
    content.push(0x00);
    content.extend_from_slice(data);
    der_tag_length_value(0x03, &content)
}

fn der_octet_string(data: &[u8]) -> Vec<u8> {
    der_tag_length_value(0x04, data)
}

fn der_generalized_time(s: &str) -> Vec<u8> {
    der_tag_length_value(0x18, s.as_bytes())
}

/// Context-specific EXPLICIT tag: `[tag_num]` EXPLICIT
fn der_explicit(tag_num: u8, content: &[u8]) -> Vec<u8> {
    der_tag_length_value(0xa0 | tag_num, content)
}

/// Context-specific IMPLICIT tag: `[tag_num]` IMPLICIT (replaces the inner tag)
fn der_implicit(tag_num: u8, content: &[u8]) -> Vec<u8> {
    der_tag_length_value(0x80 | tag_num, content)
}

fn push_length(out: &mut Vec<u8>, len: usize) {
    if len < 0x80 {
        #[expect(clippy::cast_possible_truncation, reason = "guarded by < 0x80")]
        out.push(len as u8);
    } else if len <= 0xff {
        out.push(0x81);
        #[expect(clippy::cast_possible_truncation, reason = "guarded by <= 0xff")]
        out.push(len as u8);
    } else {
        out.push(0x82);
        #[expect(clippy::cast_possible_truncation, reason = "guarded by 16-bit range")]
        {
            out.push((len >> 8) as u8);
            out.push(len as u8);
        }
    }
}

fn length_bytes(len: usize) -> usize {
    if len < 0x80 {
        1
    } else if len <= 0xff {
        2
    } else {
        3
    }
}

fn concat_slices(slices: &[&[u8]]) -> Vec<u8> {
    let total: usize = slices.iter().map(|s| s.len()).sum();
    let mut out = Vec::with_capacity(total);
    for s in slices {
        out.extend_from_slice(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_valid_der() {
        let (cert_der, key_der) = generate_self_signed_ed25519("songbird.local");
        assert!(cert_der.len() > 100, "cert too short: {}", cert_der.len());
        assert_eq!(key_der.len(), 48, "PKCS#8 Ed25519 key should be 48 bytes");
        assert_eq!(cert_der[0], 0x30, "cert must start with SEQUENCE tag");
        assert_eq!(key_der[0], 0x30, "key must start with SEQUENCE tag");
    }

    #[test]
    fn pkcs8_der_structure_valid() {
        let secret = [0x42u8; 32];
        let der = ed25519_pkcs8_der(&secret);
        assert_eq!(der.len(), 48);
        assert_eq!(&der[..2], &[0x30, 0x2e]); // SEQUENCE
        assert_eq!(&der[2..5], &[0x02, 0x01, 0x00]); // version 0
        assert_eq!(&der[5..12], &[0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70]); // Ed25519 OID
        assert_eq!(&der[16..], &secret); // key material
    }

    #[test]
    fn der_integer_adds_leading_zero_for_high_bit() {
        let val = [0x80, 0x01];
        let encoded = der_integer(&val);
        assert_eq!(encoded, vec![0x02, 0x03, 0x00, 0x80, 0x01]);
    }

    #[test]
    fn der_integer_no_padding_when_not_needed() {
        let val = [0x7f, 0x01];
        let encoded = der_integer(&val);
        assert_eq!(encoded, vec![0x02, 0x02, 0x7f, 0x01]);
    }

    #[test]
    fn different_calls_produce_different_certs() {
        let (cert1, key1) = generate_self_signed_ed25519("a.primal");
        let (cert2, key2) = generate_self_signed_ed25519("b.primal");
        assert_ne!(cert1, cert2);
        assert_ne!(key1, key2);
    }
}
