// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! RFC 9001 Section 6: Key Update mechanism for 1-RTT keys.
//!
//! After the handshake is complete, either endpoint can initiate a key update.
//! New keys are derived from the current traffic secret using HKDF-Expand-Label.

use crate::crypto::initial_keys::{hkdf_expand_label, DirectionalKeys};
use crate::crypto::provider::{QuicCipherSuite, QuicCryptoProvider};
use crate::error::Result;

/// Derive the next generation of traffic secret from the current one.
///
/// RFC 9001 Section 6:
/// ```text
/// application_traffic_secret_N+1 =
///     HKDF-Expand-Label(application_traffic_secret_N, "quic ku", "", Hash.length)
/// ```
pub async fn derive_next_secret(
    crypto: &dyn QuicCryptoProvider,
    current_secret: &[u8],
    suite: QuicCipherSuite,
) -> Result<Vec<u8>> {
    hkdf_expand_label(crypto, current_secret, b"quic ku", &[], suite.hash_len()).await
}

/// Derive new directional keys from a traffic secret.
pub async fn derive_keys_from_secret(
    crypto: &dyn QuicCryptoProvider,
    secret: &[u8],
    suite: QuicCipherSuite,
) -> Result<DirectionalKeys> {
    let key = hkdf_expand_label(crypto, secret, b"quic key", &[], suite.key_len()).await?;
    let iv = hkdf_expand_label(crypto, secret, b"quic iv", &[], suite.iv_len()).await?;
    let hp_key = hkdf_expand_label(crypto, secret, b"quic hp", &[], suite.hp_key_len()).await?;
    Ok(DirectionalKeys { key, iv, hp_key })
}

/// State for tracking key updates on a single direction.
#[derive(Debug)]
pub struct KeyUpdateState {
    /// Current traffic secret.
    current_secret: Vec<u8>,
    /// Current keys derived from the secret.
    current_keys: DirectionalKeys,
    /// Key generation number.
    generation: u64,
    /// Cipher suite.
    suite: QuicCipherSuite,
}

impl KeyUpdateState {
    /// Create from an initial traffic secret and keys.
    #[must_use]
    pub const fn new(
        secret: Vec<u8>,
        keys: DirectionalKeys,
        suite: QuicCipherSuite,
    ) -> Self {
        Self {
            current_secret: secret,
            current_keys: keys,
            generation: 0,
            suite,
        }
    }

    /// Current generation number.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Current keys.
    #[must_use]
    pub const fn keys(&self) -> &DirectionalKeys {
        &self.current_keys
    }

    /// Advance to the next key generation.
    pub async fn update(&mut self, crypto: &dyn QuicCryptoProvider) -> Result<()> {
        let next_secret = derive_next_secret(crypto, &self.current_secret, self.suite).await?;
        let next_keys = derive_keys_from_secret(crypto, &next_secret, self.suite).await?;
        self.current_secret = next_secret;
        self.current_keys = next_keys;
        self.generation += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_update_state_initial() {
        let keys = DirectionalKeys {
            key: vec![0u8; 16],
            iv: vec![0u8; 12],
            hp_key: vec![0u8; 16],
        };
        let state = KeyUpdateState::new(
            vec![0u8; 32],
            keys.clone(),
            QuicCipherSuite::Aes128Gcm,
        );
        assert_eq!(state.generation(), 0);
        assert_eq!(state.keys().key, keys.key);
    }
}
