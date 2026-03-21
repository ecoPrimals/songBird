// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Onion service descriptor generation
//!
//! **Phase 2D**: Onion Service

use crate::crypto::BeardogCryptoClient;
use crate::error::{Error, Result};
use crate::onion_service::IntroductionPoint;
use base32;

/// Attempt to get key material from `BearDog` via capability discovery.
///
/// When wired, this will perform JSON-RPC to `BearDog` over the Unix socket.
/// Until then, returns a clear error instead of silent zeroed key material.
fn request_beardog_key(method: &str) -> Result<Vec<u8>> {
    Err(Error::CryptoUnavailable(format!(
        "BearDog crypto delegation required for {method} - not yet connected"
    )))
}

/// Onion service keys (Ed25519 + X25519)
#[derive(Debug, Clone)]
pub struct OnionServiceKeys {
    /// Ed25519 identity secret key (for signing)
    pub identity_secret: [u8; 32],
    /// Ed25519 identity public key (for verification)
    pub identity_public: [u8; 32],

    /// X25519 encryption secret key (for ntor handshake)
    pub encryption_secret: [u8; 32],
    /// X25519 encryption public key (for ntor handshake)
    pub encryption_public: [u8; 32],

    /// Onion address (v3, 56 chars)
    pub onion_address: String,
}

impl OnionServiceKeys {
    /// Generate new service keys via `BearDog`
    ///
    /// # Errors
    ///
    /// Returns error if key generation or address derivation fails.
    pub async fn generate(beardog: &BeardogCryptoClient) -> Result<Self> {
        core::future::ready(()).await;
        // Ed25519 identity keypair: single BearDog RPC when wired (`secret || public`, 64 bytes)
        let identity_pair = request_beardog_key("crypto.ed25519.generate_onion_service_identity")?;
        if identity_pair.len() != 64 {
            return Err(Error::Crypto(format!(
                "BearDog onion identity keypair: expected 64 bytes, got {}",
                identity_pair.len()
            )));
        }
        let mut identity_secret = [0u8; 32];
        let mut identity_public = [0u8; 32];
        identity_secret.copy_from_slice(&identity_pair[..32]);
        identity_public.copy_from_slice(&identity_pair[32..]);

        // Generate X25519 encryption keypair
        let encryption_keypair = beardog.x25519_generate_ephemeral()?;

        // Derive onion address from public key
        let onion_address = Self::derive_onion_address(&identity_public);

        Ok(Self {
            identity_secret,
            identity_public,
            encryption_secret: encryption_keypair.secret_key,
            encryption_public: encryption_keypair.public_key,
            onion_address,
        })
    }

    /// Derive v3 onion address from Ed25519 public key
    ///
    /// Format: `base32(public_key` | checksum | version) + ".onion"
    /// - `public_key`: 32 bytes
    /// - checksum: 2 bytes (truncated SHA3-256)
    /// - version: 1 byte (0x03)
    fn derive_onion_address(public_key: &[u8; 32]) -> String {
        let version: u8 = 0x03;

        // Calculate checksum via pure Rust SHA3-256
        // checksum = SHA3-256(".onion checksum" || public_key || version)[0..2]
        let mut checksum_input = Vec::with_capacity(48);
        checksum_input.extend_from_slice(b".onion checksum");
        checksum_input.extend_from_slice(public_key);
        checksum_input.push(version);
        let hash = crate::crypto::sha3::sha3_256(&checksum_input);
        let checksum = [hash[0], hash[1]];

        // Construct address bytes (35 total)
        let mut addr_bytes = Vec::with_capacity(35);
        addr_bytes.extend_from_slice(public_key);
        addr_bytes.extend_from_slice(&checksum);
        addr_bytes.push(version);

        // Encode to base32 (56 chars)
        base32::encode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            &addr_bytes,
        )
    }
}

/// Onion service descriptor (v3)
pub struct OnionServiceDescriptor {
    /// Descriptor signing key (Ed25519)
    pub signing_key: [u8; 32],

    /// Descriptor lifetime
    pub lifetime_minutes: u32,

    /// Introduction points
    pub intro_points: Vec<IntroductionPoint>,

    /// Descriptor signature
    pub signature: Vec<u8>,
}

impl OnionServiceDescriptor {
    /// Create new descriptor
    ///
    /// # Errors
    ///
    /// Returns error if descriptor creation fails.
    pub fn new(keys: &OnionServiceKeys, intro_points: &[IntroductionPoint]) -> Result<Self> {
        // Descriptor signing uses the identity public key until blinded signing keys are modeled.
        let signing_key = keys.identity_public;

        // Default lifetime: 3 hours
        let lifetime_minutes = 180;

        // Descriptor Ed25519 signature: BearDog JSON-RPC when wired (64 bytes)
        let signature = request_beardog_key("crypto.sign.ed25519_onion_descriptor")?;
        if signature.len() != 64 {
            return Err(Error::Crypto(format!(
                "BearDog onion descriptor signature: expected 64 bytes, got {}",
                signature.len()
            )));
        }

        Ok(Self {
            signing_key,
            lifetime_minutes,
            intro_points: intro_points.to_vec(),
            signature,
        })
    }

    /// Encode descriptor for upload to `HSDir`
    ///
    /// Produces a Tor v3 descriptor in the plaintext format specified
    /// by rend-spec-v3. The descriptor has three layers:
    /// 1. Outer wrapper (plaintext, signed)
    /// 2. Superencrypted layer (encrypted to blinded key)
    /// 3. Inner encrypted layer (encrypted to subcredential)
    ///
    /// Currently produces the outer plaintext wrapper.
    /// `BearDog` integration needed for encryption layers and signing.
    ///
    /// # Errors
    ///
    /// Returns [`Error::CryptoUnavailable`] if the descriptor has no BearDog-produced signature.
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.signature.is_empty() {
            return Err(Error::CryptoUnavailable(
                "BearDog crypto delegation required: descriptor signature missing (refuse to encode with placeholder)"
                    .into(),
            ));
        }

        let mut descriptor = String::new();

        // hs-descriptor 3
        descriptor.push_str("hs-descriptor 3\n");

        // descriptor-lifetime (in minutes)
        use std::fmt::Write;
        let _ = writeln!(descriptor, "descriptor-lifetime {}", self.lifetime_minutes);

        // descriptor-signing-key-cert (placeholder)
        descriptor.push_str("descriptor-signing-key-cert\n");
        descriptor.push_str("-----BEGIN ED25519 CERT-----\n");
        // In production: BearDog generates a cross-certification
        // For now, encode the signing key as base64
        let key_b64 = base64_encode(&self.signing_key);
        descriptor.push_str(&key_b64);
        descriptor.push('\n');
        descriptor.push_str("-----END ED25519 CERT-----\n");

        // revision-counter (monotonically increasing)
        descriptor.push_str("revision-counter 1\n");

        // superencrypted (placeholder — needs BearDog encryption)
        descriptor.push_str("superencrypted\n");
        descriptor.push_str("-----BEGIN MESSAGE-----\n");
        // In production: encrypted introduction point data
        // For now, encode introduction points as plaintext
        for ip in &self.intro_points {
            let ip_b64 = base64_encode(&ip.relay_identity);
            let _ = writeln!(descriptor, "introduction-point {ip_b64}");
        }
        descriptor.push_str("-----END MESSAGE-----\n");

        // signature — must be BearDog Ed25519 (validated in `new` when using that path)
        descriptor.push_str("signature ");
        descriptor.push_str(&base64_encode(&self.signature));
        descriptor.push('\n');

        Ok(descriptor.into_bytes())
    }

    /// Calculate descriptor ID for `HSDir` lookup
    ///
    /// The descriptor ID determines which `HSDir` relays store this descriptor.
    /// Formula: `descriptor_id` = SHA3-256(signing_key || `time_period` || replica)
    ///
    /// Uses pure Rust SHA3-256 (zero `BearDog` dependency for local computation).
    /// In full Tor spec: `H(blinded_public_key` || subcredential || `time_period` || replica)
    #[must_use]
    pub fn descriptor_id(&self) -> [u8; 32] {
        let time_period = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let period_num = time_period / (u64::from(self.lifetime_minutes) * 60);

        // SHA3-256(signing_key || time_period_bytes)
        let mut input = Vec::with_capacity(40);
        input.extend_from_slice(&self.signing_key);
        input.extend_from_slice(&period_num.to_be_bytes());

        crate::crypto::sha3::sha3_256(&input)
    }
}

/// Simple base64 encoding (no external dependency)
///
/// Uses standard base64 alphabet (RFC 4648). Sufficient for descriptor encoding.
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    let chunks = data.chunks(3);

    for chunk in chunks {
        let b0 = u32::from(chunk[0]);
        let b1 = if chunk.len() > 1 {
            u32::from(chunk[1])
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            u32::from(chunk[2])
        } else {
            0
        };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_onion_address_length() {
        // v3 addresses should be 56 characters (base32 of 35 bytes)
        let public_key = [0u8; 32];
        let address = OnionServiceKeys::derive_onion_address(&public_key);

        assert_eq!(address.len(), 56);
    }

    #[test]
    fn test_descriptor_new_requires_beardog_signing() {
        let _beardog = BeardogCryptoClient::from_env().expect("Failed to create BearDog client");

        let keys = OnionServiceKeys {
            identity_secret: [0u8; 32],
            identity_public: [1u8; 32],
            encryption_secret: [2u8; 32],
            encryption_public: [3u8; 32],
            onion_address: "test".to_string(),
        };

        assert!(matches!(
            OnionServiceDescriptor::new(&keys, &[]),
            Err(Error::CryptoUnavailable(_))
        ));
    }

    #[test]
    fn test_descriptor_encoding() {
        let keys = OnionServiceKeys {
            identity_secret: [0u8; 32],
            identity_public: [1u8; 32],
            encryption_secret: [2u8; 32],
            encryption_public: [3u8; 32],
            onion_address: "test".to_string(),
        };

        // Wire-format test: non-empty signature bytes (no crypto validity asserted)
        let descriptor = OnionServiceDescriptor {
            signing_key: keys.identity_public,
            lifetime_minutes: 180,
            intro_points: vec![],
            signature: vec![0xAB; 64],
        };

        let encoded = descriptor.encode().expect("encode with test signature");
        let encoded_str = String::from_utf8_lossy(&encoded);

        // Verify descriptor format
        assert!(encoded_str.starts_with("hs-descriptor 3\n"));
        assert!(encoded_str.contains("descriptor-lifetime 180\n"));
        assert!(encoded_str.contains("descriptor-signing-key-cert\n"));
        assert!(encoded_str.contains("-----BEGIN ED25519 CERT-----\n"));
        assert!(encoded_str.contains("-----END ED25519 CERT-----\n"));
        assert!(encoded_str.contains("revision-counter 1\n"));
        assert!(encoded_str.contains("superencrypted\n"));
        assert!(encoded_str.contains("-----BEGIN MESSAGE-----\n"));
        assert!(encoded_str.contains("-----END MESSAGE-----\n"));
        assert!(encoded_str.contains("signature "));
    }

    #[test]
    fn test_descriptor_encoding_with_intro_points() {
        use crate::onion_service::IntroductionPoint;

        let keys = OnionServiceKeys {
            identity_secret: [0u8; 32],
            identity_public: [1u8; 32],
            encryption_secret: [2u8; 32],
            encryption_public: [3u8; 32],
            onion_address: "test".to_string(),
        };

        let intro_points = vec![IntroductionPoint {
            relay_identity: [0xAA; 32],
            onion_key: [0xBB; 32],
            service_key: [0xCC; 32],
            circuit_id: 1,
        }];

        let descriptor = OnionServiceDescriptor {
            signing_key: keys.identity_public,
            lifetime_minutes: 180,
            intro_points,
            signature: vec![0xCD; 64],
        };

        let encoded = descriptor.encode().expect("encode");
        let encoded_str = String::from_utf8_lossy(&encoded);

        assert!(encoded_str.contains("introduction-point "));
    }

    #[test]
    fn test_descriptor_id_deterministic_for_same_period() {
        let keys = OnionServiceKeys {
            identity_secret: [0u8; 32],
            identity_public: [1u8; 32],
            encryption_secret: [2u8; 32],
            encryption_public: [3u8; 32],
            onion_address: "test".to_string(),
        };

        let d1 = OnionServiceDescriptor {
            signing_key: keys.identity_public,
            lifetime_minutes: 180,
            intro_points: vec![],
            signature: vec![0x01; 64],
        };
        let d2 = OnionServiceDescriptor {
            signing_key: keys.identity_public,
            lifetime_minutes: 180,
            intro_points: vec![],
            signature: vec![0x01; 64],
        };

        // Same time period should produce same descriptor ID
        let id1 = d1.descriptor_id();
        let id2 = d2.descriptor_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }
}
