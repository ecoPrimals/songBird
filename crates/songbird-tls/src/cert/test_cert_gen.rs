// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Pure-Rust self-signed Ed25519 test certificates (replaces `rcgen` / `ring`).
//!
//! Mirrors the pattern in `songbird-quic::cert_gen` with optional Extended Key Usage.

use ed25519_dalek::{Signer, SigningKey};
use rand::RngCore;

/// OID for Ed25519 (1.3.101.112)
const OID_ED25519: &[u8] = &[0x2b, 0x65, 0x70];

/// OID for commonName (2.5.4.3)
const OID_CN: &[u8] = &[0x55, 0x04, 0x03];

/// OID for subjectAltName (2.5.29.17)
const OID_SAN: &[u8] = &[0x55, 0x1d, 0x11];

/// OID for extendedKeyUsage (2.5.29.37)
const OID_EKU: &[u8] = &[0x55, 0x1d, 0x25];

/// id-kp-serverAuth (1.3.6.1.5.5.7.3.1)
const OID_KP_SERVER_AUTH: &[u8] = &[0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01];

/// id-kp-clientAuth (1.3.6.1.5.5.7.3.2)
const OID_KP_CLIENT_AUTH: &[u8] = &[0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02];

/// Which key purpose OIDs to embed in Extended Key Usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestEku {
    /// TLS WWW server authentication only.
    ServerAuth,
    /// TLS client authentication only (fails `validate_purpose` when server auth is required).
    ClientAuthOnly,
}

/// Generate a self-signed Ed25519 certificate DER for tests.
#[must_use]
pub fn generate_test_ed25519_cert(
    domain: &str,
    not_before: &str,
    not_after: &str,
    eku: TestEku,
) -> Vec<u8> {
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key();

    let serial = &public_key.as_bytes()[..16];

    let tbs =
        build_tbs_certificate(serial, domain, not_before, not_after, public_key.as_bytes(), eku);
    let signature = signing_key.sign(&tbs);

    der_sequence(&[
        &tbs,
        &der_sequence(&[&der_oid(OID_ED25519)]),
        &der_bit_string(&signature.to_bytes()),
    ])
}

fn build_tbs_certificate(
    serial: &[u8],
    domain: &str,
    not_before: &str,
    not_after: &str,
    public_key: &[u8; 32],
    eku: TestEku,
) -> Vec<u8> {
    let version = der_explicit(0, &der_integer(&[2])); // v3
    let serial_num = der_integer(serial);
    let sig_alg = der_sequence(&[&der_oid(OID_ED25519)]);
    let name = build_name(domain);
    let validity =
        der_sequence(&[&der_generalized_time(not_before), &der_generalized_time(not_after)]);
    let spki =
        der_sequence(&[&der_sequence(&[&der_oid(OID_ED25519)]), &der_bit_string(public_key)]);

    let san_value = der_sequence(&[&der_implicit(2, domain.as_bytes())]);
    let san_ext = der_sequence(&[&der_oid(OID_SAN), &der_octet_string(&san_value)]);

    let kp_oid = match eku {
        TestEku::ServerAuth => OID_KP_SERVER_AUTH,
        TestEku::ClientAuthOnly => OID_KP_CLIENT_AUTH,
    };
    let eku_inner = der_sequence(&[&der_oid(kp_oid)]);
    let eku_ext = der_sequence(&[&der_oid(OID_EKU), &der_octet_string(&eku_inner)]);

    let extensions = der_explicit(3, &der_sequence(&[&san_ext, &eku_ext]));

    der_sequence(&[&version, &serial_num, &sig_alg, &name, &validity, &name, &spki, &extensions])
}

fn build_name(cn: &str) -> Vec<u8> {
    let attr = der_sequence(&[&der_oid(OID_CN), &der_utf8_string(cn)]);
    let rdn = der_set(&[&attr]);
    der_sequence(&[&rdn])
}

// --- DER encoding primitives (aligned with songbird-quic `cert_gen`) ---

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

fn der_explicit(tag_num: u8, content: &[u8]) -> Vec<u8> {
    der_tag_length_value(0xa0 | tag_num, content)
}

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
