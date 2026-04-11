// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS Server Core - Main struct and constructor
//!
//! Contains the `TlsServer` struct definition and basic initialization.

use crate::crypto::CryptoCapability;
use crate::tls::handshake_v2::keys::{CipherSuite, TrafficKeys};
use crate::tls::handshake_v2::transcript::Transcript;
use std::sync::Arc;
use tracing::info;

/// TLS 1.3 Server
///
/// Implements RFC 8446 TLS 1.3 server by reusing client components.
/// **Critical**: Uses SAME transcript logic as client for self-testing!
pub struct TlsServer {
    /// Shared crypto provider (`security provider` or any `CryptoCapability` impl)
    pub(super) crypto: Arc<dyn CryptoCapability>,

    /// Transcript tracking (SAME as client!)
    pub(super) transcript: Transcript,

    /// Server certificate chain (DER encoded)
    pub(super) cert_chain: Vec<u8>,

    /// Server private key (DER encoded)
    /// Used for certificate verification and signing (future implementation)
    #[allow(dead_code, reason = "stored for future cert signing; only read in unit tests today")]
    pub(super) private_key: Vec<u8>,

    /// Negotiated cipher suite
    pub(super) cipher_suite: CipherSuite,

    /// Handshake traffic keys
    pub(super) handshake_keys: Option<TrafficKeys>,

    /// Application traffic keys
    pub(super) application_keys: Option<TrafficKeys>,

    /// Server keypair for ECDH
    pub(super) server_private_key: Option<Vec<u8>>,
    pub(super) server_public_key: Option<Vec<u8>>,

    /// Randoms for key derivation
    pub(super) client_random: Option<Vec<u8>>,
    pub(super) server_random: Option<Vec<u8>>,

    /// Shared secret for key derivation
    pub(super) shared_secret: Option<Vec<u8>>,
}

impl TlsServer {
    /// Create new TLS server with certificate and private key
    pub fn new(
        crypto: Arc<dyn CryptoCapability>,
        cert_chain: Vec<u8>,
        private_key: Vec<u8>,
    ) -> Self {
        info!("🔐 Creating TLS 1.3 server (RFC 8446)");
        info!("   Certificate chain: {} bytes", cert_chain.len());
        info!("   Private key: {} bytes", private_key.len());

        Self {
            crypto,
            transcript: Transcript::new(),
            cert_chain,
            private_key,
            cipher_suite: CipherSuite::Aes128GcmSha256, // Default, will be negotiated
            handshake_keys: None,
            application_keys: None,
            server_private_key: None,
            server_public_key: None,
            client_random: None,
            server_random: None,
            shared_secret: None,
        }
    }

    /// Get mutable reference to transcript
    pub(super) const fn transcript_mut(&mut self) -> &mut Transcript {
        &mut self.transcript
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::SecurityCryptoProvider;

    fn create_test_crypto() -> Arc<dyn CryptoCapability> {
        let path = tempfile::env::temp_dir()
            .join("songbird-test-security.sock")
            .to_string_lossy()
            .into_owned();
        Arc::new(SecurityCryptoProvider::new(path))
    }

    #[test]
    fn test_server_creation() {
        let crypto = create_test_crypto();
        let cert = vec![1, 2, 3];
        let key = vec![4, 5, 6];

        let server = TlsServer::new(crypto, cert.clone(), key.clone());

        assert_eq!(server.cert_chain, cert);
        assert_eq!(server.private_key, key);
        assert_eq!(server.transcript.len(), 0);
    }
}
